use mini_redis::{Connection, Frame};
use std::io::{self, ErrorKind};
use tokio::net::{TcpListener, TcpStream};

async fn process(socket: TcpStream) {
    let mut connection = Connection::new(socket);
    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);
        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}

pub async fn serve(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(async move { process(socket).await });
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
