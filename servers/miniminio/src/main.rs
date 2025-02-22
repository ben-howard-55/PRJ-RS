use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection,Frame};
use std::sync::Arc;

use shared_lib::sharded_db;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6378").await.unwrap();

    println!("Miniminio Is Running!");

    let db = sharded_db::ShardedDB::new(10);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        println!("Accepted");
        let db_clone = Arc::clone(&db);
        tokio::spawn(async move {
            process(socket, db_clone).await;
        });
    }
}

async fn process(socket: TcpStream, db: Arc<sharded_db::ShardedDB>){
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(&cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await.unwrap();
    }
}