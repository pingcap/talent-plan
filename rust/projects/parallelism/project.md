# Project: Parallelism

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread-pool
- Use crossbeam channels
- Use concurrent data structures
- Perform compaction in a background thread
- Benchmark single-threaded vs multi-threaded

**Topics**: threads, thread-pools, channels, locks


## Introduction

In this project you will create a simple key/value server and client that
communicate over a custom protocol. The server will use synchronous networking,
and will respond to multiple requests in parallel by distributing work across a
thread pool of your own design. The in-memory index will become a concurrent
data structure, shared by all threads, and compaction will be done on a
dedicated thread, to reduce latency of individual requests.


## Project spec

The cargo project, `kvs`, builds a command-line key-value store client called
`kvs-client`, and a key-value store server called `kvs-server`, both of which in
turn call into a library called `kvs`. The client speaks to the server over
a custom protocol.

The interface to the CLI and the library is the same as in the previous project.
Refer to it. The difference this time is in the parallel implementation.

When setting a key to a value, `KvStore` writes the `set` command to disk in
a sequential log. On startup, the commands in the log are re-evaluated and the
log pointer (file offset) of the last command to modify each key recorded in the
in-memory index.

When retrieving a value for a key with the `get` command, it searches the index,
and if found then loads from the log, and evaluates, the command at the
corresponding log pointer.

When the size of the uncompacted log entries reach a given threshold, `KvStore`
compacts it into a new log, removing redundent entries to reclaim disk space.



## Project setup

Create a new cargo project and copy `tests/tests.rs` into it. This project
should contain a library named `kvs`, and two executables, `kvs-server` and
`kvs-client`. As with previous projects, add enough definitions that the
test suite builds.


## Part 1: Command line parsing


## Part 2: Logging


## Part 3: Client-server networking setup


## Part 4: Implementing commands across the network


## Part 5: More error handling



## Part 6: Pluggable storage engines


## Part 7: Benchmarking


## Extension 1: Signal handling


<!--

## Background reading ideas


## TODOs


-->
