use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::protocol::message::{Error, Message};

use super::message::{ARRAY_BYTE, BULK_BYTE, EOL_BYTE_ENCODING, NULL_BYTE_ENCODING, SIMPLE_BYTE};

#[derive(Debug)]
pub struct Connection {
    // The `TcpStream`. It is decorated with a `BufWriter`, which provides write
    // level buffering. The `BufWriter` implementation provided by Tokio is
    // sufficient for our needs.
    stream: BufWriter<TcpStream>,

    // The buffer for reading frames.
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            // Default to a 4KB read buffer. For the use case of mini redis,
            // this is fine. However, real applications will want to tune this
            // value to their specific use case. There is a high likelihood that
            // a larger read buffer will work better.
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_message(&mut self) -> crate::Result<Option<Message>> {
        loop {
            if let Some(message) = self.parse_message()? {
                return Ok(Some(message))
            }
            
            // On success, the number of bytes is returned. `0` indicates "end
            // of stream"
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // if no bytes are in the buffer then fine, if bytes then connection
                // was abrubtly killed.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_message(&mut self) -> crate::Result<Option<Message>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Message::check(&mut buf) {
            Ok(_) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let message = Message::parse(&mut buf)?;
                self.buffer.advance(len);
                Ok(Some(message))
            }
            Err(Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn write_message(&mut self, message: &Message) -> io::Result<()> {
        match message {
            Message::Array(val) => {
                self.stream.write_u8(ARRAY_BYTE).await?; // messge type
                self.write_decimal(val.len() as u64).await?; // arr length
                for m in &**val {
                    self.write_value(m).await?;
                } 
            }
            _ => self.write_value(message).await?
        }

        self.stream.flush().await
    }

    async fn write_value(&mut self, message: &Message) -> io::Result<()> {
        match message {
            Message::Simple(val) => {
                self.stream.write_u8(SIMPLE_BYTE).await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(EOL_BYTE_ENCODING).await?;
            }
            Message::Bulk(val) => {
                self.stream.write_u8(BULK_BYTE).await?;
                self.write_decimal(val.len() as u64).await?;
                self.stream.write_all(&val).await?;
                self.stream.write_all(EOL_BYTE_ENCODING).await?;
            }
            Message::Null => {
                self.stream.write_u8(BULK_BYTE).await?;
                self.stream.write_all(NULL_BYTE_ENCODING).await?;
                self.stream.write_all(EOL_BYTE_ENCODING).await?;
            }
            // Encoding an `Array` from within a value cannot be done using a
            // recursive strategy. In general, async fns do not support
            // recursion.
            Message::Array(_val) => unreachable!(),
        }

        Ok(())
    }

    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        let mut buf = [0u8; 20];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;

        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.stream.write_all(b"\r\n").await?;

        Ok(())
    }
}
