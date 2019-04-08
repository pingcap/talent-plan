use crate::engines::KvsEngine;
use crate::{KvStore, Result};
use lock_api::Mutex;
use parking_lot::RawMutex;

/// The `KvStore` engine
pub struct KvStoreEngine {
    // TODO: Use channel and thread pool in KvStore to eliminate the lock?
    store: Mutex<RawMutex, KvStore>,
}

impl KvStoreEngine {
    /// Create a `KvStoreEngine`
    pub fn new(kvs: KvStore) -> Self {
        KvStoreEngine {
            store: Mutex::new(kvs),
        }
    }
}

impl KvsEngine for KvStoreEngine {
    fn set(&self, key: String, value: String) -> Result<()> {
        self.store.lock().set(key, value)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        self.store.lock().get(key)
    }
}
