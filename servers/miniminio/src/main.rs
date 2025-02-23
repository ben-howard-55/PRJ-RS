use miniminio::protocol::connection::Connection;
use tokio::net::{TcpListener, TcpStream};
// use mini_redis::{Connection,Frame};
use std::sync::Arc;

use shared_lib::{client_model::{DataStoreServiceSchema, ObjectLocation}, sharded_db};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6378").await.unwrap();

    println!("Miniminio Is Running!");

    // TODO: probably in the future want to create owner threads and channels that own said thread?
    let data_store = sharded_db::ShardedDB::<DataStoreServiceSchema>::new(10);
    let object_store = sharded_db::ShardedDB::<ObjectLocation>::new(10);

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        println!("Accepted");
        let data_store_clone = Arc::clone(&data_store);
        let object_store_clone = Arc::clone(&object_store);
        tokio::spawn(async move {
            process(socket, data_store_clone, object_store_clone).await;
        });
    }
}

async fn process(socket: TcpStream, data_store: Arc<sharded_db::ShardedDB::<DataStoreServiceSchema>>, object_store: Arc<sharded_db::ShardedDB::<ObjectLocation>>){
    use mini_redis::Command::{self, Get, Set};

    // let mut connection = Connection::new(socket);
    let mut connection = Connection::new(socket);

    while let Some(message) = connection.read_message().await.unwrap() {
        // let response = match Command::from_frame(frame).unwrap() {
            // Set(cmd) => {
            //     db.insert(&cmd.key().to_string(), cmd.value().clone());
            //     Frame::Simple("OK".to_string())
            // }
            // Get(cmd) => {
            //     if let Some(value) = db.get(cmd.key()) {
            //         Frame::Bulk(value.clone())
            //     } else {
            //         Frame::Null
            //     }
            // }
            // cmd => panic!("unimplemented {:?}", cmd),
        // };
        // println!(&message);
        connection.write_message(&message).await.unwrap();
    }
}