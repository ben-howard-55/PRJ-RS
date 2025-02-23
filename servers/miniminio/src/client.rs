use shared_lib::client_model::UploadId;
use tokio::net::ToSocketAddrs;
use tokio::net::TcpStream;
use crate::{operations::create_mutlipart_upload::CreateMultipartUploadRequest, protocol::{connection::Connection, message}};


pub struct MiniMinioClient {
    connection: Connection,
}

impl MiniMinioClient {
    pub async fn create_mutlipart_upload(&mut self, bucket: &str, key: &str, version: &str) -> crate::Result<UploadId> {
        let mpu= CreateMultipartUploadRequest::new(bucket, key, version);
        let message = mpu.to_message();
        self.connection.write_message(&message).await?;
       
        let upload_id = uuid::Uuid::new_v4().to_string();
        return Ok(upload_id);

        // let res = self.connection.read_message().await?;
    }

}


pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<MiniMinioClient> {
    let socket = TcpStream::connect(addr).await.unwrap();
    let connection = Connection::new(socket);

    Ok(MiniMinioClient { connection })
}