pub struct KvStore;

impl KvStore {
    /// Creates a `KvStore`.
    pub fn new() -> KvStore {
        unimplemented!()
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) {
        unimplemented!()
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&self, key: String) -> Option<String> {
        unimplemented!()
    }

    /// Removes a given key.
    pub fn remove(&mut self, key: String) {
        unimplemented!()
    }
}
