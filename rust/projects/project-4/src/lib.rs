//! A simple key/value store.

pub use engines::{KvStore, KvsEngine};
pub use error::{KvsError, Result};

mod engines;
mod error;
pub mod thread_pool;
