use super::ThreadPool;
use crate::Result;

/// Wrapper of rayon::ThreadPool
pub struct RayonThreadPool;

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self> {
        unimplemented!()
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        unimplemented!()
    }
}
