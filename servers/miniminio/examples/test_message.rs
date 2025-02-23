use miniminio::message::Message;
use miniminio::connection::Connection;
use tokio::net::TcpStream;

const ADDR: &str = "127.0.0.1:6378";

#[tokio::main]
async fn main() {
    let socket = TcpStream::connect(ADDR).await.unwrap();
    let mut connection = Connection::new(socket);
    let message = Message::Simple(("Hello World".to_string()));
    // message.push_bulk(Bytes::from("hello_world".as_bytes()));

    let _ = connection.write_message(&message).await;

    let response = connection.read_message().await;
    println!("returned");
    println!("{:?}", response.unwrap());
}