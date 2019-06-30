pub use self::kvs::KvStore;
use crate::Result;

mod kvs;

/// Trait for a key value storage engine.
pub trait KvsEngine: Clone + Send + 'static {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    fn set(&self, key: String, value: String) -> Result<()>;

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// Returns a `KvsError` if the given key is not found.
    fn remove(&self, key: String) -> Result<()>;
}
