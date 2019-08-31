use std::path::PathBuf;

use crate::Result;

pub struct KvStore;

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        unimplemented!()
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        unimplemented!()
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// Returns a `KvsError` if the given key is not found.
    pub fn remove(&mut self, key: String) -> Result<()> {
        unimplemented!()
    }
}