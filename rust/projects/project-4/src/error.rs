/// Error type for kvs
#[derive(Debug)]
pub enum KvsError {
}

/// Result type for kvs
pub type Result<T> = std::result::Result<T, KvsError>;
