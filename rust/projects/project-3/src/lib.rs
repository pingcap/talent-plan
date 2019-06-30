//! A simple key/value store.

pub use engines::{KvStore, KvsEngine};
pub use error::Result;

mod engines;
mod error;