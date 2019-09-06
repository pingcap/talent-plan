#![feature(integer_atomics)]
#![deny(clippy::all)]
// You need to remove these two allows.
#![allow(dead_code)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
extern crate prost_derive;

mod kvraft;
mod proto;
mod raft;

/// A place holder for suppressing unused_variables warning.
fn your_code_here<T>(_: T) -> ! {
    unimplemented!()
}
