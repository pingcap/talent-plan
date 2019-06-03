use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use kvs::thread_pool::*;
use kvs::Result;

use crossbeam_utils::sync::WaitGroup;

fn spawn_counter<P: ThreadPool>() -> Result<()> {
    const TASK_NUM: usize = 20;
    const ADD_COUNT: usize = 1000;

    let pool = P::new(4)?;
    let wg = WaitGroup::new();
    let counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..TASK_NUM {
        let counter = Arc::clone(&counter);
        let wg = wg.clone();
        pool.spawn(move || {
            for _ in 0..ADD_COUNT {
                counter.fetch_add(1, Ordering::Relaxed);
            }
            drop(wg);
        })
    }

    wg.wait();
    assert_eq!(counter.load(Ordering::SeqCst), TASK_NUM * ADD_COUNT);
    Ok(())
}

fn panic_task<P: ThreadPool>() -> Result<()> {
    const TASK_NUM: usize = 1000;

    let pool = P::new(4)?;
    for _ in 0..TASK_NUM {
        pool.spawn(move || {
            panic!();
        })
    }

    spawn_counter::<P>()
}

#[test]
fn naive_thread_pool_spawn_counter() -> Result<()> {
    spawn_counter::<NaiveThreadPool>()
}

#[test]
fn shared_queue_thread_pool_spawn_counter() -> Result<()> {
    spawn_counter::<SharedQueueThreadPool>()
}

#[test]
fn rayon_thread_pool_spawn_counter() -> Result<()> {
    spawn_counter::<RayonThreadPool>()
}

#[test]
fn shared_queue_thread_pool_panic_task() -> Result<()> {
    panic_task::<SharedQueueThreadPool>()
}
