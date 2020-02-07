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

use std::sync::mpsc::{channel, Receiver};
use std::thread::sleep;
use std::time::Duration;

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
/// to describe our logic, uses the sync function `$handler` to pass the rpc.
/// This macro requires `self` has a struct named `rpc_execution_pool`.
/// This will spawn a new thread on `rpc_execution_pool` each time the rpc endpoint called.
/// TODO: allow rename to this pool.
#[macro_export]
macro_rules! async_rpc {
    ($name:ident($arg:ty) -> $rel:ty where uses $handler:expr) => {
        fn $name(&self, args: $arg) -> RpcFuture<$rel> {
            let (sx, rx) = futures::sync::oneshot::channel();
            let myself = self.clone();
            self.rpc_execution_pool.spawn(move || {
                sx.send($handler(&myself, args))
                    .unwrap_or_else(|args| {
                        warn!(
                            concat!("RPC channel exception, RPC", stringify!($name), "({:?})won't be replied."),
                            args
                        )
                    });
            });
            Box::new(rx.map_err(|e| panic!("rpc: failed to execute: {}", e)))
        }
    }
}
