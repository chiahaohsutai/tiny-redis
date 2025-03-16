use bytes::Bytes;
use mini_redis::Command::{self, Get, Set};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use std::hash::{self, Hash, Hasher};

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = hash::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub struct ShardedDB {
    db: Arc<Vec<Mutex<HashMap<String, Bytes>>>>,
}

impl ShardedDB {
    pub fn new(n: usize) -> Self {
        let mut db = Vec::with_capacity(n);
        (0..n).for_each(|_| db.push(Mutex::new(HashMap::new())));
        ShardedDB { db: Arc::new(db) }
    }
    pub fn insert(&self, key: &str, value: Bytes) {
        let mut shard = self.db[self.index(&key)].lock().unwrap();
        shard.insert(key.to_string(), value);
    }
    pub fn get(&self, key: &str) -> Option<Bytes> {
        let shard = self.db[self.index(&key)].lock().unwrap();
        shard.get(key).cloned()
    }
    fn index(&self, key: &str) -> usize {
        (hash(&key) % self.db.len() as u64) as usize 
    }
}

async fn process(socket: TcpStream, db: Arc<ShardedDB>) {
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap_or(None) {
        let resp = match Command::from_frame(frame) {
            Ok(command) => {
                match command {
                    Get(cmd) => db
                        .get(cmd.key())
                        .map(|v| Frame::Bulk(v.clone()))
                        .unwrap_or(Frame::Null),
                    Set(cmd) => {
                        db.insert(cmd.key(), cmd.value().clone());
                        Frame::Simple(String::from("Ok"))
                    }
                    _ => Frame::Error(String::from("Unsupported command.")),
                }
            }
            Err(err) => {
                eprintln!("Error parsing command from frame: {}", err);
                Frame::Error(String::from(String::from("Unsupported command.")))
            }
        };
        connection.write_frame(&resp).await.unwrap_or(());
    }
}

pub async fn serve(listener: TcpListener, database: Arc<ShardedDB>) {
    loop {
        let database = database.clone();
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(async move { process(socket, database).await });
            }
            Err(e) => {
                let err = &format!("Failed to accept client connection: {}", e);
                let description = match e.kind() {
                    ErrorKind::ConnectionRefused => "Connection was refused.",
                    ErrorKind::ConnectionReset => "Connection was reset.",
                    ErrorKind::TimedOut => "Connection timed out.",
                    _ => "Check server logs for more details.",
                };
                eprint!("{err} - {description}")
            }
        };
    }
}
