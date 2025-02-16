use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use std::sync::{Arc, Mutex};
use std::collections::{HashMap};

use bytes::Bytes;

type ShardedMap = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

#[derive(Clone)]
pub struct ShardedDB {
    db: ShardedMap,
}

impl ShardedDB {
    pub fn new(num_shards: usize) -> Arc<Self> {
        let mut db =  Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db.push(Mutex::new(HashMap::new()));
        }

        Arc::new(Self { db: Arc::new(db), })
    }

    pub fn insert(&self, key: &str, value: Bytes) {
        let shard_index = self.get_key_shard(key);
        let mut shard = self.db[shard_index].lock().unwrap();
        shard.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        let shard_index = self.get_key_shard(key);
        let shard = self.db[shard_index].lock().unwrap();
        shard.get(key).cloned()
    }

    fn get_key_shard(&self, key: &str) -> usize{
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        (s.finish() as usize) % self.db.len()
    }
}