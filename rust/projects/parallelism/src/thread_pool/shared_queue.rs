use std::{panic, thread};

use super::ThreadPool;
use crate::Result;

use crossbeam::channel::{self, Receiver, Sender};

// Note for Rust training course: the thread pool is not implemented using
// `catch_unwind` because it would require the task to be `UnwindSafe`.

/// A thread pool using a shared queue inside.
///
/// If a spawned task panics, the old thread will be destroyed and a new one will be
/// created. It fails silently when any failure to create the thread at the OS level
/// is captured after the thread pool is created. So, the thread number in the pool
/// can decrease to zero, then spawning a task to the thread pool will panic.
pub struct SharedQueueThreadPool {
    tx: Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<Self> {
        let (tx, rx) = channel::unbounded::<Box<dyn FnOnce() + Send + 'static>>();
        for _ in 0..threads {
            let rx = rx.clone();
            thread::Builder::new().spawn(move || run_tasks(rx))?;
        }
        Ok(SharedQueueThreadPool { tx })
    }

    /// Spawns a function into the thread pool.
    ///
    /// # Panics
    ///
    /// Panics if the thread pool has no thread.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx
            .send(Box::new(job))
            .expect("The thread pool has no thread.");
    }
}

fn run_tasks(rx: Receiver<Box<dyn FnOnce() + Send + 'static>>) {
    let rx2 = rx.clone();
    panic::set_hook(Box::new(move |panic_info| {
        error!("Thread panicked: {}", panic_info);
        let rx = rx2.clone();
        if let Err(e) = thread::Builder::new().spawn(move || run_tasks(rx)) {
            error!("Failed to spawn a thread: {}", e);
        }
    }));

    loop {
        match rx.recv() {
            Ok(task) => {
                task();
            }
            Err(_) => info!("Thread exits because the thread pool is destroyed."),
        }
    }
}
