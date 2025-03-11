use bytes::Bytes;
use mini_redis::Command::{self, Get, Set};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type Database = Arc<Mutex<HashMap<String, Bytes>>>;

async fn process(socket: TcpStream, database: Database) {
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap_or(None) {
        let resp = match Command::from_frame(frame) {
            Ok(command) => {
                let mut db = database.lock().unwrap();
                match command {
                    Get(cmd) => db
                        .get(cmd.key())
                        .map(|v| Frame::Bulk(v.clone()))
                        .unwrap_or(Frame::Null),
                    Set(cmd) => {
                        db.insert(String::from(cmd.key()), cmd.value().clone());
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

pub async fn serve(listener: TcpListener, database: Database) {
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
