#![feature(integer_atomics)]
#![feature(duration_as_u128)]

extern crate labcodec;
extern crate labrpc;
extern crate prost;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prost_derive;
extern crate futures;
extern crate rand;

mod kvraft;
mod raft;
