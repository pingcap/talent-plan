use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::PathBuf;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in a log. A `HashMap` in memory
/// stores the keys and the value locations for fast query.
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
    // directory for the log and other data
    #[allow(dead_code)]
    path: PathBuf,
    log_reader: BufReaderWithPos<File>,
    log_writer: BufWriterWithPos<File>,
    // stores keys and the pos of the last command to modify each
    index: HashMap<String, CommandPos>,
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
        create_dir_all(&path)?;

        let log_path = path.join("data.log");

        let mut log_writer = BufWriterWithPos::new(
            OpenOptions::new()
                .create(true)
                .read(true)
                .append(true)
                .open(&log_path)?,
        )?;
        // Set pos to end of file
        log_writer.seek(SeekFrom::End(0))?;

        let log_reader = BufReaderWithPos::new(File::open(&log_path)?)?;

        let mut store = KvStore {
            path,
            log_reader,
            log_writer,
            index: HashMap::new(),
        };
        store.load_from_log()?;
        Ok(store)
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    ///
    /// # Error
    ///
    /// It propagates I/O or serialization errors during writing the log.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = SetCommand::new(key, value);
        let pos = self.log_writer.pos;
        serde_json::to_writer(&mut self.log_writer, &cmd)?;
        self.log_writer.flush()?;
        self.index
            .insert(cmd.key, (pos..self.log_writer.pos).into());
        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            self.log_reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = (&mut self.log_reader).take(cmd_pos.len);
            let set_cmd: SetCommand = serde_json::from_reader(cmd_reader)?;
            Ok(Some(set_cmd.value))
        } else {
            Ok(None)
        }
    }

    fn load_from_log(&mut self) -> Result<()> {
        let mut pos = self.log_reader.seek(SeekFrom::Start(0))?;
        let mut stream = Deserializer::from_reader(&mut self.log_reader).into_iter::<SetCommand>();
        while let Some(set_cmd) = stream.next() {
            let new_pos = stream.byte_offset() as u64;
            self.index.insert(set_cmd?.key, (pos..new_pos).into());
            pos = new_pos;
        }
        Ok(())
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
