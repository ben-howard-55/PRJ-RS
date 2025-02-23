use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc::channel;
use tokio::sync::oneshot::{self, Sender};

const ADDR: &str = "127.0.0.1:6379";

type Responder<T> = Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();
        let cmd = Command::Get { key: ("foo".to_string()), resp: res_tx};
        tx.send(cmd).await.unwrap();
        
        let res = res_rx.await;
        print!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (res_tx, res_rx) = oneshot::channel();
        let cmd = Command::Set { key: ("foo".to_string()), val: ("bar".into()) , resp: res_tx};
        tx2.send(cmd).await.unwrap();

        let res = res_rx.await.unwrap();
        print!("GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {

        let mut client = client::connect(ADDR).await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp} => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set {key, val, resp} => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}