use crate::protocol::message::Message;

use bytes::Bytes;
use core::str;
use std::{fmt, vec};

pub(crate) struct MessageParser {
    parts: vec::IntoIter<Message>,
}

#[derive(Debug)]
pub(crate) enum ParserError {
    EndOfStream,
    Other(crate::Error),
}

impl MessageParser {
    pub(crate) fn new(message: Message) -> Result<MessageParser, ParserError> {
        let array = match message {
            Message::Array(array) => array,
            message => return Err(format!("Protocol Error; expected array message, got {:?}", message).into()),
        };

        Ok(MessageParser {
            parts: array.into_iter(),
        })
    }

    fn next(&mut self) -> Result<Message, ParserError> {
        self.parts.next().ok_or(ParserError::EndOfStream)
    }

    pub(crate) fn next_string(&mut self) -> Result<String, ParserError> {
        match self.next()? {
            Message::Simple(S) => Ok(S),
            Message::Bulk(data) => str::from_utf8(&data[..])
                .map(|s| s.to_string())
                .map_err(|_| "protocol error; invalid string".into()),
            message => Err(format!(
                "protocol error; expected simple message or bulk message, got {:?}",
                message
            )
            .into()),
        }
    }

    pub(crate) fn next_bytes(&mut self) -> Result<Bytes, ParserError> {
        match self.next()? {
            Message::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            Message::Bulk(data) => Ok(data),
            message => Err(format!(
                "protocol error; expected simple message or bulk message, got {:?}",
                message
            )
            .into()),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), ParserError> {
        if self.parts.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of message".into())
        }
    }
}


impl From<String> for ParserError {
    fn from(src: String) -> ParserError {
        ParserError::Other(src.into())
    }
}

impl From<&str> for ParserError {
    fn from(src: &str) -> ParserError {
        src.to_string().into()
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParserError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParserError {}
