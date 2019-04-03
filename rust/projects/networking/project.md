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

- `kvs-client shutdown [--addr IP-PORT]`

  Shutdown the server. Wait for shutdown to complete or error before returning.

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
`kvs-client`.

_Do that now._


## Part 1: Command line parsing

## Part 2: Logging

- add log or sled and a console logger
- log address and engine on startup

## Part 3: Client-server networking setup

- listen and connect, log events

## Part 4: Implementing commands across the network

- create a network protocol
- implement get, set, shutdown

## Part 5: More error handling

- handle error responses by converting errors to a serializable format
- add context to errors
- replace `fn main() -> Result` with custom error reporting

## Part 6: Pluggable backends

- abstract engine into a trait
- add sled as an engine
- error when incorrect engine is selected, per previous data

## Part 7: Benchmarking

- add criterion benchmark suite
- add identical benchmarks for "kvs" and "sled" engines

## Extension 1: Signal handling

- Shutdown on KILL

<!--

## TODOs


-->
