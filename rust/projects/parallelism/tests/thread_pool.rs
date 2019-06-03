use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

use kvs::thread_pool::*;
use kvs::Result;

#[test]
fn naive_thread_pool_spawn_counter() -> Result<()> {
    const THREAD_NUM: usize = 20;
    const ADD_COUNT: usize = 1000;

    let pool = NaiveThreadPool::new(4)?;
    let barrier = Arc::new(Barrier::new(THREAD_NUM + 1));
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..THREAD_NUM {
        let counter = Arc::clone(&counter);
        let barrier = Arc::clone(&barrier);
        pool.spawn(move || {
            for _ in 0..ADD_COUNT {
                counter.fetch_add(1, Ordering::Relaxed);
            }
            barrier.wait();
        })
    }

    barrier.wait();
    assert_eq!(counter.load(Ordering::SeqCst), THREAD_NUM * ADD_COUNT);
    Ok(())
}
