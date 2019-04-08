#![deny(missing_docs)]
//! A simple key/value store.

#[macro_use]
extern crate log;

pub use client::KvsClient;
pub use engines::kvs::KvStoreEngine;
pub use engines::KvsEngine;
pub use error::{KvsError, Result};
pub use kv::KvStore;
pub use server::KvsServer;

mod client;
mod common;
mod engines;
mod error;
mod kv;
mod server;
