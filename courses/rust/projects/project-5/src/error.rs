use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Error type for kvs
#[derive(Error, Debug)]
pub enum KvsError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// Serialization or deserialization error
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Removing non-existent key error
    #[error("Key not found")]
    KeyNotFound,
    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    #[error("Unexpected command type")]
    UnexpectedCommandType,
    /// Key or value is invalid UTF-8 sequence
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    /// Sled error
    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),
    /// Error with a string message
    #[error("{0}")]
    StringError(String),
}

/// Result type for kvs
pub type Result<T> = std::result::Result<T, KvsError>;
