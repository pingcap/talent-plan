use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam_skiplist::SkipMap;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use super::KvsEngine;
use crate::{KvsError, Result};
use std::ffi::OsStr;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in log files. Log files are named after
/// monotonically increasing generation numbers with a `log` extension name.
/// A `BTreeMap` in memory stores the keys and the value locations for fast query.
///
/// ```rust
/// # use kvs::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// use kvs::KvsEngine;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owned())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
pub struct KvStore {
    path: Arc<PathBuf>,
    // map generation number to the file reader
    reader: KvStoreReader,
    index: Arc<SkipMap<String, CommandPos>>,
    writer: Arc<Mutex<KvStoreWriter>>,
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        KvStore {
            path: Arc::clone(&self.path),
            reader: KvStoreReader::new(Arc::clone(&self.path)),
            index: Arc::clone(&self.index),
            writer: Arc::clone(&self.writer),
        }
    }
}

struct KvStoreReader {
    path: Arc<PathBuf>,
    readers: RefCell<HashMap<u64, BufReaderWithPos<File>>>,
}

impl KvStoreReader {
    fn new(path: Arc<PathBuf>) -> KvStoreReader {
        KvStoreReader {
            path,
            readers: RefCell::new(HashMap::new()),
        }
    }

    fn read_and<F, R>(&self, cmd_pos: CommandPos, f: F) -> Result<R>
    where
        F: FnOnce(io::Take<&mut BufReaderWithPos<File>>) -> Result<R>,
    {
        let mut readers = self.readers.borrow_mut();
        // We don't use entry API here because we want the errors to be propogated.
        if !readers.contains_key(&cmd_pos.gen) {
            let reader = BufReaderWithPos::new(File::open(log_path(&self.path, cmd_pos.gen))?)?;
            readers.insert(cmd_pos.gen, reader);
        }
        let reader = readers.get_mut(&cmd_pos.gen).unwrap();
        reader.seek(SeekFrom::Start(cmd_pos.pos))?;
        let cmd_reader = reader.take(cmd_pos.len);
        f(cmd_reader)
    }

    fn read_command(&self, cmd_pos: CommandPos) -> Result<Command> {
        self.read_and(cmd_pos, |cmd_reader| {
            Ok(serde_json::from_reader(cmd_reader)?)
        })
    }
}

struct KvStoreWriter {
    writer: BufWriterWithPos<File>,
    current_gen: u64,
    uncompacted: u64,
    path: Arc<PathBuf>,
    index: Arc<SkipMap<String, CommandPos>>,
}

impl KvStore {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// # Errors
    ///
    /// It propagates I/O or deserialization errors during the log replay.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = Arc::new(path.into());
        fs::create_dir_all(&*path)?;

        let mut readers = HashMap::new();
        let index = Arc::new(SkipMap::new());

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &*index)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&0) + 1;
        let writer = new_log_file(&path, current_gen)?;

        let writer = KvStoreWriter {
            writer,
            current_gen,
            uncompacted,
            path: Arc::clone(&path),
            index: Arc::clone(&index),
        };

        let reader = KvStoreReader::new(Arc::clone(&path));

        Ok(KvStore {
            path,
            reader,
            index,
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

impl KvsEngine for KvStore {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Errors
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            if let Command::Set { value, .. } = self.reader.read_command(*cmd_pos.value())? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a given key.
    ///
    /// # Error
    ///
    /// It returns `KvsError::KeyNotFound` if the given key is not found.
    ///
    /// It propagates I/O or serialization errors during writing the log.
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}

impl KvStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.get(&key) {
                self.uncompacted += old_cmd.value().len;
            }
            self.index
                .insert(key, (self.current_gen, pos..self.writer.pos).into());
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            let pos = self.writer.pos;
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("key not found");
                self.uncompacted += old_cmd.value().len;
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                self.uncompacted += self.writer.pos - pos;
            }

            if self.uncompacted > COMPACTION_THRESHOLD {
                self.compact()?;
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    /// Clears stale entries in the log.
    fn compact(&mut self) -> Result<()> {
        // increase current gen by 2. current_gen + 1 is for the compaction file
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = new_log_file(&self.path, self.current_gen)?;

        let mut compaction_writer = new_log_file(&self.path, compaction_gen)?;

        let mut new_pos = 0; // pos in the new log file
        let reader = KvStoreReader::new(Arc::clone(&self.path));
        let mut stale_gens = HashSet::new();
        for entry in self.index.iter() {
            let len = reader.read_and(*entry.value(), |mut entry_reader| {
                Ok(io::copy(&mut entry_reader, &mut compaction_writer)?)
            })?;
            stale_gens.insert(entry.value().gen);
            self.index.insert(
                entry.key().clone(),
                (compaction_gen, new_pos..new_pos + len).into(),
            );
            new_pos += len;
        }
        compaction_writer.flush()?;

        // remove stale log files
        for stale_gen in stale_gens {
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }
        self.uncompacted = 0;

        Ok(())
    }
}

/// Create a new log file with given generation number and add the reader to the readers map.
///
/// Returns the writer to the log.
fn new_log_file(path: &Path, gen: u64) -> Result<BufWriterWithPos<File>> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;
    Ok(writer)
}

/// Returns sorted generation numbers in the given directory
fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    gen_list.sort_unstable();
    Ok(gen_list)
}

/// Load the whole log file and store value locations in the index map.
///
/// Returns how many bytes can be saved after a compaction.
fn load(
    gen: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &SkipMap<String, CommandPos>,
) -> Result<u64> {
    // To make sure we read from the beginning of the file
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0; // number of bytes that can be saved after a compaction
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.get(&key) {
                    uncompacted += old_cmd.value().len;
                }
                index.insert(key, (gen, pos..new_pos).into());
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.value().len;
                }
                // the "remove" command itself can be deleted in the next compaction
                // so we add its length to `uncompacted`
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
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
#[derive(Debug, Clone, Copy)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen,
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
