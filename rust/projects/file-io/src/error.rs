use failure::Fail;
use std::io;

/// Error type for kvs
#[derive(Fail, Debug)]
pub enum KvsError {
    /// Wraps a `std::io::Error`.
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

/// Result type for kvs
pub type Result<T> = std::result::Result<T, KvsError>;
