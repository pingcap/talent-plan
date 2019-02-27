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
#[macro_use]
extern crate log;
#[macro_use]
extern crate prost_derive;
extern crate futures;
extern crate futures_timer;
extern crate rand;

mod kvraft;
mod raft;
