# Project: Synchronous client-server networking

**Task**: Create a single-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Create a client-server application
- Write a custom protocol with `std` networking APIs
- Introduce logging to the server
- Chain errors and report them in a human-readable way
- Implement pluggable backends via traits
- Benchmark the hand-written backend against `sled`

**Topics**: `std::net`, logging, error handling, `impl Trait`, benchmarking

**Extensions**: shutdown on signal


## Introduction

In this project you will create a simple key/value server and client. They will
communicate with a custom networking protocol of your design. You will emit logs
using standard logging crates, and handle errors correctly across the network
boundary. Once you have a working client-server architecture,
then you will abstract the storage engine behind traits, and compare
the performance of yours to the [`sled`] engine.

## Project spec

The cargo project, `kvs`, builds a command-line key-value store client called
`kvs-client`, and a key-value store server called `kvs-server`, both of which in
turn call into a library called `kvs`. The client speaks to the server over
a custom protocol.

The `kvs-server` executable supports the following command line arguments:

- `kvs-server [--addr IP-PORT] [--engine ENGINE-NAME]`

  Start the server and begin listening for incoming connections. `--addr`
  accepts an IP address, either v4 or v6, and a port number, with the format
  `IP:PORT`. If `--addr` is not specified then listen on `127.0.0.1:4000`.

  If `--engine` is specified, then `ENGINE-NAME` must be either "kvs", in which
  case the built-in engine is used, or "sled", in which case sled is used. If
  this is the first run (there is no data previously persisted) then the default
  value is "kvs"; if there is previously persisted data then the default is the
  engine already in use. If data was previously persisted with a different
  engine than selected, print an error and exit with a non-zero exit code.

  Print an error and return a non-zero exit code on failure to bind a socket, if
  `ENGINE-NAME` is invalid, if `IP-PORT` does not parse as an address.

- `kvs-server -v`

  Print the version.

The `kvs-client` executable supports the following command line arguments:

- `kvs-client set <KEY> <VALUE> [--addr IP-PORT]`

  Set the value of a string key to a string.

  `--addr` accepts an IP address, either v4 or v6, and a port number, with the
  format `IP:PORT`. If `--addr` is not specified then connect on
  `127.0.0.1:4000`.

  Print an error and return a non-zero exit code on server error,
  or if `IP-PORT` does not parse as an address.

- `kvs-client get <KEY> [--addr IP-PORT]`

  Get the string value of a given string key.

  `--addr` accepts an IP address, either v4 or v6, and a port number, with the
  format `IP:PORT`. If `--addr` is not specified then connect on
  `127.0.0.1:4000`.

  Print an error and return a non-zero exit code on server error,
  or if `IP-PORT` does not parse as an address.

- `kvs-client -V`

  Print the version.

The `kvs` library contains three types: `KvsClient`, `KvsServer`, and
`KvsStore`. `KvsClient` implements the functionality required for `kvs-client`
to speak to `kvs-server`; `KvsServer` implements the functionality to serve
responses to `kvs-client` from `kvs-server`; the `KvsEngine` trait
defines the storage interface used by `KvsServer`; `KvStore` implements
by hand the `KvsEngine` trait, and `SledKvStore` implements `KvsEngine`
for the [`sled`] storage engine.

[`sled`]: https://github.com/spacejam/sled

The `KvsEngine` trait supports the following methods:

- `KvsEngine::set(key: String, value: String) -> Result<()>`

  Set the value of a string key to a string.

  Return an error if the value is not written successfully.

- `KvsEngine::get(key: String) -> Result<Option<String>>`

  Get the string value of the a string key.
  If the key does not exist, return `None`.

  Return an error if the value is not read successfully.

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

There's little new about the command line parsing in this project compared to
previous projects. The `kvs-client` binary accepts the same command line
arguments as in previous projects. And now `kvs-server` has its own set of
command line arguments to handle.


## Part 2: Logging

Production server applications tend to have robust and configurable logging. So
now we're going to add logging to `tikv-server`, and as we continue will look
for useful information to log. During development it is common to use logging
at the `debug!` and `trace!` levels for "println debugging".

There are two prominent logging systems in Rust: [`log`] and [`slog`]. Both
export similar macros for logging at different levels, like `error!`, `info!`
etc. Both are extensible, supporting different backends, for logging to the
console, logging to file, logging to the system log, etc.

[`log`]: https://docs.rs/log/
[`slog`]: https://docs.rs/slog/

The major difference is that `log` is fairly simple, logging only formatted
strings; `slog` is feature-rich, and supports "structured logging", where log
entries are typed and serialized in easily-parsed formats.

`log` dates from the very earliest days of Rust, where it was part of the
compiler, then part of the standard library, and finally released as its own
crate. It is maintained by the Rust Project. `slog` is newer and maintained
independently. Both are widely used.

For both systems, one needs to select a "sink" crate, one that the logger
sends logs to for display or storage.

_Read about both of them, choose the one that appeals to you, add them as
dependencies, then modify `kvs-server` to initialize logging on startup, prior
to command-line parsing._ Set it up to output to stderr (sending the logs
elsewhere additionally is fine, but they must go to stderr to pass the tests in
this project).

On startup log the server's version number. Also log the configuration. For now
that means the IP address and port, and the name of the storage engine.


## Part 3: Client-server networking setup

Next we're going to set up the networking. For this project you are going to be
using the basic TCP/IP networking APIs in `std::net`: `TcpListener` and
`TcpStream`. Before thinking about the protocol, modify `kvs-server` to listen
for and accept connections, and `kvs-client` to initiate connections.

For this project, the server is synchronous and single-threaded. That means that
you will listen on a socket, then accept connections, and execute and respond to
commands one at a time. In the future we will re-visit this decision multiple
times on our journey toward an asynchronous, multi-threaded, and
high-performance database.

Thank about your manual testing workflow. Now that there are two executables to
deal with, you'll need a way to run them both at the same time. If you are like
many, you will use two terminals, running `cargo run --bin --tikv-server` in
one, where it runs until you press CTRL-D`, and `cargo run --bin --tikv-client`
in the other.

This is a good opportunity to use the logging macros for debugging. Go ahead and
log information about every accepted connection.


## Part 4: Implementing commands across the network

Now it's time to implement the key/value store over the network, remotely
executing commands that until now have been implemented within a single process.
As with the file I/O you worked on in the last project to create the log, you
will be serializing and streaming commands with the `Read` and `Write` traits.

You are going to design a network protocol.

TODO: Can't think of much to say about this


## Part 5: More error handling

- handle error responses by converting errors to a serializable format
- add context to errors
- replace `fn main() -> Result` with custom error reporting

## Part 6: Pluggable storage engines

Your database has a storage engine, `KvStore`, implemented by you.
Now you are going to add a second storage engine.

There are multiple reasons to do so:

- Different workloads require difference performance characteristics. Some
  storage engines may work better than other based on the workload.

- It creates a familiar framework for comparing different backends.

- It gives us an excuse to create and work with traits.

- It gives us an excuse to write some comparative benchmarks!

So you are going to _extract_ a new trait, `KvsEngine`, from the `KvStore`
interface. This is a classic _refactoring_, where existing code is transformed
into a new form incrementally. When refactoring you will generally want to break
the work up into the smallest changes that will continue to build and work.

Here is the API you need to end up with:

- `trait KvsEngine` has `get` and `set` methods with the same signatures
  as `KvStore`.

- `KvStore` implements `KvsEngine`, and no longer has `get` and `set` methods of
  its own.

- There is a free function, `pub fn create_engine(name: &str) -> impl
  KvsEngine`, that constructs a concrete implementation of `KvsEngine`.

- There is a new implementation of `KvsEngine`, `SledKvsEngine`. It can
  be constructed with `create_engine`.

It's likely that you have already stubbed out the definitions for these if your
tests are building. _Now is the time to fill them in._ Break down your
refactoring into an intentional sequence of changes, and make sure the project
continues to build and pass previously-passing tests before continuing.

As one final step, you need to consider what happens when `tikv-server` is
started with one engine, is killed, then restarted with a different engine. This
case can only result in an error, and you need to figure out how to detect the
case to report the error. The test `cli_wrong_engine` reflects this scenario.


## Part 7: Benchmarking

- add criterion benchmark suite
- add identical benchmarks for "kvs" and "sled" engines

## Extension 1: Signal handling

- Shutdown on KILL

<!--

## Background reading

- log docs
- slog docs
- TCP/IP basics
- refactoring overview
- traits and impl trait

## TODOs

- consider `Kvs_Engine_` trait vs `Kv_Store_` impl

-->
