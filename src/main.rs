use mini_redis::{Connection, Frame};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::{env, process};
use tokio::net::{TcpListener, TcpStream};
use tiny_redis::serve;

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .unwrap_or(String::from("6379"))
        .parse::<u16>()
        .unwrap();

    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port);
    let listener = TcpListener::bind(addr).await;

    match listener {
        Ok(listener) => serve(listener).await,
        Err(err) => {
            let e = &format!("Failed to bind to socket address {}: {}", addr, err);
            let description = match err.kind() {
                ErrorKind::AddrInUse => "The port is already in use.",
                ErrorKind::AddrNotAvailable => "The address is not available.",
                ErrorKind::PermissionDenied => "Permission denied while binding.",
                _ => "Check server logs for more details.",
            };
            eprintln!("{e} - {description}");
            process::exit(1);
        }
    }
}
