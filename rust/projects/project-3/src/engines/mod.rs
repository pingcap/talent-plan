//! This module provides various key value storage engines.

use crate::Result;

/// Trait for a key value storage engine.
pub trait KvsEngine {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// Returns a `KvsError` if the given key is not found.
    fn remove(&mut self, key: String) -> Result<()>;
}

mod kvs;

pub use self::kvs::KvStore;
