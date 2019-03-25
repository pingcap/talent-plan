extern crate labcodec;
extern crate prost_derive;
#[macro_use]
extern crate labrpc;

// After you finish the implementation, `#[allow(unused)]` should be removed.
#[allow(dead_code, unused)]
mod client;
#[allow(unused)]
mod server;
mod service;
#[cfg(test)]
mod tests;

// This is related to protobuf as described in `msg.proto`.
mod msg {
    include!(concat!(env!("OUT_DIR"), "/msg.rs"));
}
