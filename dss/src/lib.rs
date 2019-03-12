#![feature(integer_atomics)]
#![feature(duration_as_u128)]
#![deny(clippy::all)]
// You need to remove these two allows.
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate labcodec;
extern crate labrpc;
extern crate linearizability;
extern crate prost;
#[allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate prost_derive;
#[cfg(test)]
extern crate env_logger;
extern crate futures;
extern crate futures_timer;
extern crate rand;

mod kvraft;
mod raft;
