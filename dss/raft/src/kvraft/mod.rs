//! The key/value service based on raft service.
//!
//! # kvraft(Lab 3)
//! ## general
//! We start a state machine `KvStateMachine` that receive commands from raft.
//!
//! We can start command parallel, by a `Waker` table named `waiting_channels` in the `KvStateMachine`.
//!
//! We check duplicated request by a table named `last_command` in the `KvStateMachine`.
//!
//! `KvStateMachine` starts by `new`, it spawns a worker thread, which receiving `ApplyMsg` from raft node,
//! wake up waiting threads, and transform state of itself.
//!
//! `Clerk` starts by `request`, which is a generic method that doing leader check, retry, timeout, etc..
//!
//! ## where to find algorithm implementation
//! ### Key/Value service based on raft (3A)
//! `KvStateMachine::new` starts all.
//!
//! `Node` handles the rpc, and is the client of `KvStateMachine`.
//! Code at `do_*` functions shows how it deal with rpc.
//!
//! ### Raft with snapshot (3B)
//!
//! `RaftLogWithSnapshot` is the modified log struct for snapshot, which seems like a vector with offset.
//!
//! `raft::Node::take_snapshot` interface for client to take a new snapshot.
//!
//! `raft::Node::do_install_snapshot` for InstallSnapshot rpc.
//!

pub mod client;
#[cfg(test)]
pub mod config;
pub mod errors;
pub mod server;
#[cfg(test)]
mod tests;
