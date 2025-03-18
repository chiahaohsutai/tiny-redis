use bytes::Buf;
use std::io::Cursor;
use bytes::BytesMut;
use mini_redis::{Frame, Result};
use mini_redis::frame::Error::Incomplete;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let buffer = BytesMut::with_capacity(4096);
        let stream = BufWriter::new(stream);
        Connection { stream, buffer }
    }
    pub async fn read(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse().await? {
                return Ok(Some(frame));
            };
            if self.stream.read_buf(&mut self.buffer).await? == 0 {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("Connection reset by peer.".into());
                }
            };
        }
    }
    pub async fn parse(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(frame))
            },
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
