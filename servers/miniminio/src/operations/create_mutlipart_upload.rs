use bytes::Bytes;

use crate::protocol::{message::Message, parser::MessageParser};

#[derive(Clone, Debug)]
pub struct CreateMultipartUploadRequest {
    bucket: String,
    key: String,
    version: String,
}

impl CreateMultipartUploadRequest {

    pub fn new(bucket: impl ToString, key: impl ToString, version: impl ToString) -> CreateMultipartUploadRequest {
        CreateMultipartUploadRequest {
            bucket: bucket.to_string(),
            key: key.to_string(),
            version: version.to_string(),
        }
    }

    pub(crate) fn parse_message(parse: &mut MessageParser) -> crate::Result<CreateMultipartUploadRequest> {
        // CreateMultipartUploadRequest has already been consumed.
        let bucket = parse.next_string()?;
        let key = parse.next_string()?;
        let version = parse.next_string()?;

        Ok(CreateMultipartUploadRequest{bucket, key, version})
    } 

    pub(crate) fn to_message(self) -> Message {
        let mut message = Message::array();
        message.push_bulk(Bytes::from("CreateMultiPartUpload".as_bytes()));
        message.push_bulk(Bytes::from(self.bucket.into_bytes()));
        message.push_bulk(Bytes::from(self.key.into_bytes()));
        message.push_bulk(Bytes::from(self.version.into_bytes()));
        message
    }
}