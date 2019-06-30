use std::path::PathBuf;
use super::KvsEngine;
use crate::thread_pool::ThreadPool;
use crate::{KvsError, Result};

use tokio::prelude::*;

#[derive(Clone)]
pub struct KvStore<P: ThreadPool> {
    thread_pool: P
}

impl<P: ThreadPool> KvStore<P> {
    /// Opens a `KvStore` with the given path.
    ///
    /// This will create a new directory if the given one does not exist.
    ///
    /// `concurrency` specifies how many threads at most can read the database at the same time.
    pub fn open(path: impl Into<PathBuf>, concurrency: u32) -> Result<Self> {
        unimplemented!()
    }
}

impl<P: ThreadPool> KvsEngine for KvStore<P> {
    fn set(&self, key: String, value: String) -> Box<Future<Item = (), Error = KvsError> + Send> {
        unimplemented!()
    }

    fn get(&self, key: String) -> Box<Future<Item = Option<String>, Error = KvsError> + Send> {
        unimplemented!()
    }

    fn remove(&self, key: String) -> Box<Future<Item = (), Error = KvsError> + Send> {
        unimplemented!()
    }
}
