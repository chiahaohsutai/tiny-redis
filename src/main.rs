use mini_redis::{Connection, Frame};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{env, process};
use tokio::net::{TcpListener, TcpStream};

fn handle_client_conn_err(error: io::Error) {
    let e = &format!("Failed to accept client connection: {}", error);
    let description = match error.kind() {
        ErrorKind::ConnectionRefused => "Connection was refused.",
        ErrorKind::ConnectionReset => "Connection was reset.",
        ErrorKind::TimedOut => "Connection timed out.",
        _ => "Check server logs for more details.",
    };
    eprint!("{e} - {description}")
}

fn handle_server_bind_err(addr: SocketAddrV4, error: io::Error) {
    let e = &format!("Failed to bind to socket address {}: {}", addr, error);
    let description = match error.kind() {
        ErrorKind::AddrInUse => "The port is already in use.",
        ErrorKind::AddrNotAvailable => "The address is not available.",
        ErrorKind::PermissionDenied => "Permission denied while binding.",
        _ => "Check server logs for more details.",
    };
    eprintln!("{e} - {description}");
}

async fn process_client_conn(socket: TcpStream) {
    let mut connection = Connection::new(socket);
    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);
        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}

async fn handle_client_conns(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((socket, _)) => process_client_conn(socket).await,
            Err(e) => handle_client_conn_err(e),
        };
    }
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .unwrap_or(String::from("6379"))
        .parse::<u16>()
        .unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let listener = TcpListener::bind(addr).await;

    match listener {
        Ok(listener) => handle_client_conns(listener).await,
        Err(e) => {
            handle_server_bind_err(addr, e);
            process::exit(1);
        }
    }
}
