#![feature(integer_atomics)]
#![feature(box_syntax)]
#![feature(todo_macro)]
#![deny(clippy::all)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
extern crate prost_derive;

use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::sleep;
use std::time::Duration;

use rayon::ThreadPool;

use crate::TimerMsg::{Reset, Stop};

pub mod kvraft;
mod proto;
pub mod raft;

/// A place holder for suppressing unused_variables warning.
fn your_code_here<T>(_: T) -> ! {
    unimplemented!()
}

fn select<T: Send + 'static, I: Iterator<Item = Receiver<T>> + Send + 'static>(
    channels: I,
) -> Receiver<T> {
    let (sx, rx) = channel();
    std::thread::spawn(move || {
        let recv = select_idx(channels);
        while let Ok(data) = recv.recv() {
            if sx.send(data.1).is_err() {
                return;
            }
        }
    });
    rx
}

// 有没有比轮询更加好的办法呢？（似乎 Go 语言的 Select 在规模变大之后使用的也是轮询）
fn select_idx<T: Send + 'static, I: Iterator<Item = Receiver<T>>>(
    channels: I,
) -> Receiver<(usize, T)> {
    use std::sync::mpsc::TryRecvError::*;
    let (sx, rx) = channel();
    let channels: Vec<Receiver<T>> = channels.collect();
    let mut is_available = vec![true; channels.len()];
    let mut available_count = channels.len();
    std::thread::spawn(move || loop {
        if available_count == 0 {
            return;
        }
        for (i, ch) in channels.iter().enumerate() {
            if is_available[i] {
                match ch.try_recv() {
                    Ok(data) => {
                        if let Err(_e) = sx.send((i, data)) {
                            debug!("select: select receiver is closed.");
                            return;
                        }
                    }
                    Err(Disconnected) => {
                        is_available[i] = false;
                        available_count -= 1;
                    }
                    Err(Empty) => {}
                }
            }
        }
        sleep(Duration::from_millis(8))
    });
    rx
}

/// declare a rpc endpoint, that instead of uses async functions(i.e. functions in the future context)
/// to describe our logic, uses the sync function `$handler` to handle the rpc.
/// This will spawn a new thread each time the rpc endpoint called.
#[macro_export]
macro_rules! async_rpc {
    ($name:ident($arg:ty) -> $rel:ty where uses $handler:expr) => {
        fn $name(&self, args: $arg) -> RpcFuture<$rel> {
            let (sx, rx) = futures::sync::oneshot::channel();
            let myself = self.clone();
            std::thread::spawn(move || {
                sx.send($handler(&myself, args))
                    .unwrap_or_else(|args| {
                        warn!(
                            concat!("RPC channel exception, RPC", stringify!($name), "({:?})won't be replied."),
                            args
                        )
                    });
            });
            Box::new(rx.map_err(|_| labrpc::Error::Stopped))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
enum TimerMsg {
    Reset,
    Stop,
}

/// A timer that will do specified task when time out.
/// You can reset the timer to prevent this.
/// And you can stop the timer when don't need it any more.
#[derive(Clone)]
struct Timer {
    sx: Sender<TimerMsg>,
}

#[allow(dead_code)]
impl Timer {
    /// create a new timer.
    /// the timer after `timeout_gen()`, do `time_up()`.
    /// each time it is reset or fire a time up signal,
    pub fn new(
        timeout_gen: impl Fn() -> Duration + Send + 'static,
        time_up: impl Fn() + Send + 'static,
    ) -> Self {
        use std::sync::mpsc::RecvTimeoutError::*;
        let (sx, rx) = channel();
        std::thread::spawn(move || loop {
            let message = rx.recv_timeout(timeout_gen());
            match message {
                Err(Timeout) => {
                    time_up();
                }
                Ok(TimerMsg::Reset) => debug!("Timer: timeout."),
                Ok(TimerMsg::Stop) | Err(Disconnected) => {
                    debug!("stop signal received, the timer will stop.");
                    return;
                }
            }
        });
        Timer { sx }
    }

    /// reset the timer.
    ///
    /// # returns
    /// if success to reset the timer, return `true`.
    pub fn reset(&self) -> bool {
        self.sx.send(Reset).is_ok()
    }

    /// stop the timer.
    ///
    /// # returns
    /// if success to stop the timer, return `true`.
    pub fn stop(&self) -> bool {
        self.sx.send(Stop).is_ok()
    }
}

// we make sure that only call to `terminate` breaks the borrow rule.
unsafe impl Sync for ThreadPoolWithDrop {}

struct ThreadPoolWithDrop {
    t: RefCell<Option<ThreadPool>>,
}

impl Into<ThreadPoolWithDrop> for ThreadPool {
    fn into(self) -> ThreadPoolWithDrop {
        ThreadPoolWithDrop {
            t: RefCell::new(Some(self)),
        }
    }
}

impl ThreadPoolWithDrop {
    fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        let b = self.t.try_borrow();
        if b.is_err() {
            warn!("spawn on a dropping pool -- nop.");
            return;
        }
        let p = b.unwrap();
        if p.is_none() {
            warn!("spawn on a dropping pool -- nop.");
            return;
        }
        p.as_ref().unwrap().spawn(f)
    }

    unsafe fn terminate(&self) {
        drop(self.t.borrow_mut().take())
    }

    #[allow(dead_code)]
    fn is_terminated(&self) -> bool {
        let b = self.t.try_borrow();
        if b.is_err() {
            return true;
        }
        b.unwrap().is_none()
    }
}
