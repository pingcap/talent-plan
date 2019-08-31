use std::path::PathBuf;
use super::KvsEngine;
use crate::Result;

pub struct KvStore;

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        unimplemented!()
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        unimplemented!()
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    fn remove(&mut self, key: String) -> Result<()> {
        unimplemented!()
    }
}