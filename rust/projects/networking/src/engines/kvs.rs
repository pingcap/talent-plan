use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use super::KvsEngine;
use crate::{KvsError, Result};
use std::ffi::OsStr;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in a log. A `HashMap` in memory
/// stores the keys and the value locations for fast query.
///
/// Example:
///
/// ```rust
/// # use kvs::{KvStore, KvsEngine};
/// use std::env::current_dir;
/// let store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok::<(), kvs::KvsError>(())
/// ```
pub struct KvStore {
    // directory for the log and other data
    path: PathBuf,
    // In this iteration, KvStore is used in a single thread,
    // so we just use RefCell to get mutable access to KvLog.
    // This will be replaced with a channel and a background working thread
    // in the next iteration.
    kv_log: RefCell<KvLog>,
    log_gen: Cell<u64>,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Error
    ///
    /// It propagates I/O or deserialization errors during the log replay.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        let log_gen = latest_gen(&path)?;
        let mut kv_log = KvLog::open(path.join(format!("{}.log", log_gen)))?;
        kv_log.load()?;

        Ok(KvStore {
            path,
            kv_log: RefCell::new(kv_log),
            log_gen: Cell::new(log_gen),
        })
    }

    /// Clears stale entries in the log.
    pub fn compact(&self) -> Result<()> {
        let log_gen = self.log_gen.get();
        let new_path = self.path.join(format!("{}.log", log_gen + 1));

        // Create lock file that indicates compaction is in progress.
        let lock_path = new_path.with_extension("lock");
        drop(File::create(&lock_path)?);

        let kv_log = self.kv_log.borrow_mut().compact(new_path)?;

        // Compaction is all over. Remove the lock file.
        fs::remove_file(&lock_path)?;

        let old_kv_log = self.kv_log.replace(kv_log);
        self.log_gen.set(log_gen + 1);

        // Close old log file before removing it. (It's a must on Windows I think)
        let old_path = old_kv_log.path.clone();
        // The old file is useless. It's safe we just drop it.
        drop(old_kv_log);
        fs::remove_file(old_path)?; // TODO: Maybe this error can be just ignored?

        Ok(())
    }
}

impl KvsEngine for KvStore {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Error
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.kv_log.borrow_mut().set(key, value)?;
        if self.kv_log.borrow().uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        self.kv_log.borrow_mut().get(key)
    }

    /// Removes a given key.
    ///
    /// # Error
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn remove(&self, key: String) -> Result<()> {
        self.kv_log.borrow_mut().remove(key)
    }
}

struct KvLog {
    path: PathBuf,
    reader: BufReaderWithPos<File>,
    writer: BufWriterWithPos<File>,
    // stores keys and the pos of the last command to modify each
    index: BTreeMap<String, CommandPos>,
    // the number of bytes representing "stale" commands that could be
    // deleted during a compaction
    uncompacted: u64,
    loaded: bool,
}

impl KvLog {
    // Pay attention that it does not load the log file automatically
    fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let mut writer =
            BufWriterWithPos::new(OpenOptions::new().create(true).append(true).open(&path)?)?;
        // Because file mode is set to append, we need to set pos to end of file manually to keep synced
        writer.seek(SeekFrom::End(0))?;

        let reader = BufReaderWithPos::new(File::open(&path)?)?;

        Ok(KvLog {
            path,
            reader,
            writer,
            index: BTreeMap::new(),
            uncompacted: 0,
            loaded: false,
        })
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        assert!(self.loaded);

        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.insert(key, (pos..self.writer.pos).into()) {
                self.uncompacted += old_cmd.len;
            }
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        assert!(self.loaded);

        if let Some(cmd_pos) = self.index.get(&key) {
            self.reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = (&mut self.reader).take(cmd_pos.len);
            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        assert!(self.loaded);

        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("key not found");
                self.uncompacted += old_cmd.len;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    fn load(&mut self) -> Result<()> {
        let mut pos = self.reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(&mut self.reader).into_iter::<Command>();
        while let Some(cmd) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            match cmd? {
                Command::Set { key, .. } => {
                    if let Some(old_cmd) = self.index.insert(key, (pos..new_pos).into()) {
                        self.uncompacted += old_cmd.len;
                    }
                }
                Command::Remove { key } => {
                    if let Some(old_cmd) = self.index.remove(&key) {
                        self.uncompacted += old_cmd.len;
                    }
                    // the "remove" command itself can be deleted in the next compaction
                    // so we add its length to `uncompacted`
                    self.uncompacted += new_pos - pos;
                }
            }
            pos = new_pos;
        }
        self.loaded = true;
        Ok(())
    }

    // Merge old entries to `new_path`. Returns the new KvLog.
    pub fn compact(&mut self, new_path: impl Into<PathBuf>) -> Result<KvLog> {
        // The new log file for merged entries
        let log_path = new_path.into();
        let mut new_writer = BufWriter::new(
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&log_path)?,
        );

        let mut new_pos = 0; // pos in the new log file
        let mut new_index = BTreeMap::new(); // index map for the new log file
        for (key, cmd_pos) in &self.index {
            if self.reader.pos != cmd_pos.pos {
                self.reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = (&mut self.reader).take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut new_writer)?;
            new_index.insert(key.clone(), (new_pos..new_pos + len).into());
            new_pos += len;
        }
        new_writer.flush()?;
        drop(new_writer);

        // Reopen using the new file name
        let mut kv_log = KvLog::open(&log_path)?;
        // Use the index map built on writing instead of reloading the log file
        kv_log.index = new_index;
        kv_log.loaded = true;

        Ok(kv_log)
    }
}

/// Struct representing a command
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// Represents the position and length of a json-serialized command in the log
struct CommandPos {
    pos: u64,
    len: u64,
}

impl From<Range<u64>> for CommandPos {
    fn from(range: Range<u64>) -> Self {
        CommandPos {
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

const INIT_GEN: u64 = 1;

// Log files are named after a generation number with a "log" extension name.
// Log file with a lock file indicates a compaction failure and is invalid.
// This function finds the latest valid generation number.
fn latest_gen(dir: impl AsRef<Path>) -> Result<u64> {
    let latest: Option<u64> = fs::read_dir(&dir)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| {
            path.is_file()
                && path.extension() == Some("log".as_ref())
                && !path.with_extension("lock").exists()
        })
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .max();
    Ok(latest.unwrap_or(INIT_GEN))
}
