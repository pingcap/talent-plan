pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
use crate::KvsError;

use tokio::prelude::Future;

mod kvs;
mod sled;

/// Trait for a key value storage engine.
pub trait KvsEngine: Clone + Send + 'static {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    fn set(&self, key: String, value: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send>;

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Box<dyn Future<Item = Option<String>, Error = KvsError> + Send>;

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    fn remove(&self, key: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send>;
}
