use bytes::{Buf, Bytes};
use std::convert::TryInto;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

// const INTEGER_BYTE: u8 = b':';
pub const BULK_BYTE: u8 = b'$';
pub const SIMPLE_BYTE: u8 = b'+';
pub const ERROR_BYTE: u8 = b'-';
pub const ARRAY_BYTE: u8 = b'*';

pub const EOL_BYTE_ENCODING: &[u8; 2] = b"\r\n";
pub const NULL_BYTE_ENCODING: &[u8; 2] = b"-1";

#[derive(Clone, Debug)]
pub enum Message {
    Simple(String),
    Bulk(Bytes),
    Null,
    Array(Vec<Message>),
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,
    /// Invalid message encoding
    InvalidEncoding,
    // 
    Other(crate::Error),
}

impl Message {
    pub(crate) fn array() -> Message {
        Message::Array(vec![])
    }

    pub(crate) fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Message::Array(vec) => {
                vec.push(Message::Bulk(bytes));
            }
            _ => panic!("not an array frame"),
        }
    }

    pub fn check (src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            SIMPLE_BYTE => {
                get_line(src)?;
                Ok(())
            }
            ARRAY_BYTE => {
                let len = get_decimal(src)?;

                for _ in 0..len {
                    Message::check(src)?;
                }

                Ok(())
            }
            BULK_BYTE => {
                if ERROR_BYTE == peek_u8(src)? {
                    // Skip '-1\r\n'
                    skip(src,NULL_BYTE_ENCODING.len() + EOL_BYTE_ENCODING.iter().len())
                } else {
                    let len: usize = get_decimal(src)?.try_into()?;
                    // skip that number of bytes + 2 (\r\n).
                    skip(src, len + EOL_BYTE_ENCODING.len())
                }
            }
            actual => Err(format!("protocol error; invalid message type byte `{}`", actual).into()),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Message, Error> {
        match get_u8(src)? {
            SIMPLE_BYTE => {
                let line = get_line(src)?.to_vec();
                let string = String::from_utf8(line)?;

                Ok(Message::Simple(string))
            }
            ARRAY_BYTE => {
                let len = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Message::parse(src)?);
                }

                Ok(Message::Array(out))
            }
            BULK_BYTE => {
                if ERROR_BYTE == peek_u8(src)? {
                    let line = get_line(src)?;

                    if line != NULL_BYTE_ENCODING {
                        return Err("protocol error; invalid frame format".into());
                    }
    
                    Ok(Message::Null)
                } else {
                    let len = get_decimal(src)?.try_into()?;
                    let n = len + 2;

                    if src.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);
                    
                    // skip that number of bytes + 2 (\r\n).
                    skip(src, n)?;
                    
                    Ok(Message::Bulk(data))

                }

            }
            _ => unimplemented!(),
        }
    }
}


fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

/// Read a new-line terminated decimal
fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

/// Find a line
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Message::Simple(res) => res.fmt(fmt),
            Message::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, " ")?;
                        part.fmt(fmt)?;
                    }
                }
                Ok(())
            },
            Message::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Message::Null => "null".fmt(fmt)
        }
    }
}

// TODO: understand these errors a bit better.
impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::InvalidEncoding => "invalid encoding".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}