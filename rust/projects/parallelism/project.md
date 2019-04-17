# Project: Parallelism

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread-pool
- Use crossbeam channels
- Share data structures with locks
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

The interface to the CLI and the library is the same as in the [previous project].
Refer to it. The difference this time is in the parallel implementation, which
will be described as we work through it.


## Project setup

Create a new cargo project and copy `tests/tests.rs` into it. This project
should contain a library named `kvs`, and two executables, `kvs-server` and
`kvs-client`. As with previous projects, add enough definitions that the
test suite builds.


## Part 1: Creating a thread pool

- discuss thread pool designs
- specify a simple interface
- pass test cases

## Part 2: Accepting connections on separate threads

- accept and respond to requests in the thread pool
- this will require sharing the index as well

## Part 3: Abstract thread pools

- like with the sled comparison, abstract the thread
  pool, and add an existing one
- can we do a different _kind_ of abstraction than we did for sled? maybe lets
  try compile time abstraction and cargo features

## Part 4: Comparing thread pools

- read-centric workload, measuring network throughput
- is just writing more criterion benchmarks useful?
- can we come up with other tools to measure with,
  or other reasons to measure?

## Part 5: Background compaction

- discuss issues with files and concurrency
- move compaction to a background thread
- need to refactor previous projects to use multiple logs
- this should be fairly challenging


<!--

## Background reading ideas


## TODOs

- a concurrent map or skiplist would be better than a mutexed hashmap but there
  doesn't seem to be a prod-quality crate for it
- is there some new kind of measurement we can do
  for thread pools in addition to criterion benchmarks?

-->
