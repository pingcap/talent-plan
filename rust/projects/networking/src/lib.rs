#![deny(missing_docs)]
//! A simple key/value store.

#[macro_use]
extern crate log;

pub use client::KvsClient;
pub use engines::{KvStore, KvsEngine};
pub use error::{KvsError, Result};
pub use server::KvsServer;

mod client;
mod common;
mod engines;
mod error;
mod server;
