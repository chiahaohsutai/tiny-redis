use bytes::Bytes;
use mini_redis;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let port = env::var("PORT")
        .unwrap_or(String::from("6379"))
        .parse::<u16>()
        .unwrap_or(6379);
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);

    let manager = tokio::spawn(async move {
        let mut client = mini_redis::client::connect(addr).await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let value = client.get(&key).await;
                    let _ = resp.send(value);
                }
                Command::Set { key, value, resp } => {
                    let value = client.set(&key, value).await;
                    let _ = resp.send(value);
                }
            };
        }
    });

    let task1 = tokio::spawn(async move {
        println!("Running task 1 ...");
        let (resp, rx) = oneshot::channel();
        let cmd = Command::Get {
            key: String::from("foo"),
            resp,
        };
        if tx.send(cmd).await.is_err() {
            eprintln!("Connection task shutdown");
            return;
        }
        let res = rx.await;
        println!("TASK1 GOT (GET) {:?}", res);
        println!("TASK1 Completed")
    });
    let task2 = tokio::spawn(async move {
        println!("Running task 2 ...");
        let (resp, rx) = oneshot::channel();
        let cmd = Command::Set {
            key: String::from("foo"),
            value: "bar".into(),
            resp,
        };
        if tx2.send(cmd).await.is_err() {
            eprintln!("Connection task shutdown");
            return;
        }
        let res = rx.await;
        println!("TASK2 GOT (SET) {:?}", res);
        println!("TASK2 Completed")
    });

    task1.await.unwrap();
    task2.await.unwrap();
    manager.await.unwrap();
}
