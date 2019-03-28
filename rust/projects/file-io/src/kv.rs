use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::{to_writer, Deserializer};
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
pub struct KvStore {
    // directory for wal and other data
    #[allow(dead_code)]
    path: PathBuf,
    wal_writer: BufWriter<File>,
    map: HashMap<String, String>,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        create_dir_all(&path)?;

        let mut wal_path = path.clone();
        wal_path.push("wal.log");
        let wal = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&wal_path)?;

        Ok(KvStore {
            path,
            wal_writer: BufWriter::new(wal),
            map: KvStore::load_from_wal(&wal_path)?,
        })
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = SetCommand::new(key, value);
        to_writer(&mut self.wal_writer, &cmd)?;
        self.map.insert(cmd.key, cmd.value);
        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    fn load_from_wal(wal_path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        let reader = BufReader::new(File::open(wal_path)?);
        let stream = Deserializer::from_reader(reader).into_iter::<SetCommand>();
        for set_cmd in stream {
            let set_cmd = set_cmd?;
            map.insert(set_cmd.key, set_cmd.value);
        }
        Ok(map)
    }
}

/// A struct representing the set command
#[derive(Serialize, Deserialize, Debug)]
struct SetCommand {
    key: String,
    value: String,
}

impl SetCommand {
    fn new(key: String, value: String) -> SetCommand {
        SetCommand { key, value }
    }
}
