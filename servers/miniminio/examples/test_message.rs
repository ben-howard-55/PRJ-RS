use miniminio::client::{self, MiniMinioClient};
use miniminio::protocol::message::Message;
use miniminio::protocol::connection::Connection;
use tokio::net::TcpStream;

const ADDR: &str = "127.0.0.1:6378";

#[tokio::main]
async fn main() {
    let mut minio_client = client::connect(ADDR).await.unwrap();
    let upload_id = minio_client.create_mutlipart_upload("bucket", "key", "version").await.unwrap();
    println!("{}", upload_id);

    // let response = connection.read_message().await;
    // println!("returned");
    // println!("{:?}", response.unwrap());
}