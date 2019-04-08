use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and also persisted to disk using a WAL.
///
/// Example:
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
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
    ///
    /// # Error
    ///
    /// It propagates I/O or deserialization errors during the WAL replay.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        create_dir_all(&path)?;

        let wal_path = path.join("wal.log");
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
    ///
    /// # Error
    ///
    /// It propagates I/O or serialization errors during writing the WAL.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = SetCommand::new(key, value);
        serde_json::to_writer(&mut self.wal_writer, &cmd)?;
        self.wal_writer.flush()?;
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
