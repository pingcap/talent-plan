use std::io;
use thiserror::Error;

/// Error type for kvs.
#[derive(Error, Debug)]
pub enum KvsError {
    /// IO error.
    #[error("`{0}`")]
    Io(#[from] io::Error),
    /// Serialization or deserialization error.
    #[error("`{0}`")]
    Serde(#[from] serde_json::Error),
    /// Removing non-existent key error.
    #[error("Key not found")]
    KeyNotFound,
    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    #[error("Unexpected command type")]
    UnexpectedCommandType,
}

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;
