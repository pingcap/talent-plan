# Project: File I/O

**Task**: Create a persistent key/value store that can be accessed from the
command line

**Goals**:

- Handle and report errors robustly
- Write data to disk as a write-ahead log using standard file APIs.
- Read the state of the key/value store from disk
- Use serde for serialization
- Use file I/O APIs to binary search

**Topics**: `failure` crate, `std::net::fs`, `Read` / `Write` traits,
serde

**Extensions**: range queries, store data using bitcast algo?


## Introduction

In this project you will create a simple on-disk key/value store that can be
modified and queried from the command line. You will start by maintaining a
[write-ahead log][wal] (WAL) on disk of previous commands that is loaded into
memory before fulfilling each command. Then you will extend that by periodically
converting the WAL to a searchable on-disk format. At the end of this project
you will have built a simple database using Rust file APIs.

[wal]: https://en.wikipedia.org/wiki/Write-ahead_logging

## Project spec

The cargo project, `kvs`, builds a command-line key-value store client called
`kvs`, which in turn calls into a library called `kvs`.

The `kvs` executable supports the following command line arguments:

- `kvs set <KEY> <VALUE>`

  Set the value of a string key to a string.
  Print an error and return a non-zero exit code on failure.

- `kvs get <KEY>`

  Get the string value of a given string key.
  Print an error and return a non-zero exit code on failure.

- `kvs -V`

  Print the version

The `kvs` library contains a type, `KvStore`, that supports the following
methods:

- `KvStore::set(key: String, value: String) -> Result<()>`

  Set the value of a string key to a string.
  Return an error if the value is not written successfully.

- `KvStore::get(key: String) -> Result<Option<String>>`

  Get the string value of the a string key.
  If the key does not exist, return `None`.
  Return an error if the value is not read successfully.

When setting a key to a value, `kvs` writes the `set` command to disk in a
write-ahead log. On startup, the commands in the WAL are re-evaluated and the
state cached in memory.

When the size of the WAL reaches a given threshold, `kvs` writes it to disk in a
searchable format and clears the WAL. When retrieving a value for a key with the
`get` command, first the in-memory WAL cache is searched, and if not found, then
the on-disk values are searched.

Note that our `kvs` project is both a stateless command-line program, and a
library containing a stateful `KvStore` type, and thus have different
requirements.


## Project setup

Create a new cargo project and copy `tests/tests.rs` into it. Like project 1,
this project should contain a library and an executable, both named `kvs`.

As with the previous project, use `clap` or `structopt` to handle the command
line arguments. They are the same as last time.

_Do that now._


## Part 1: Error handling

In this project it will be possible for the code to fail due to I/O errors. So
before we get started implementing a database we need to do one more thing that
is crucial to Rust projects: decide on an error handling strategy.

Rust's error handling is powerful, but involves a lot of boilerplate to use
correctly. For this project the [`failure`] crate will provide the tools to
easily handle errors of all kinds.

_Find the latest version of the failure crate and add it to your dependencies in
`Cargo.toml`._

[`failure`]: https://docs.rs/failure/0.1.5/failure/

The [failure guide][fg] describes [several] error handling patterns.

[fg]: https://boats.gitlab.io/failure/
[several]: https://boats.gitlab.io/failure/guidance.html

Pick one of those strategies and, in your library, either define your own error
type or import `failure`s `Error`. This is the error type you will use in all of
your `Result`s, converting error types from other crates to your own with the
`?` operator.

After that, define a type alias for `Result` that includes your concrete error
type, so that you don't need to type `Result<T, YourErrorType>` everywhere, but
can simply type `Result<T>`. This type alias is also typically called `Result`,
[shadowing] the standard library's.

[shadowing]: https://en.wikipedia.org/wiki/Variable_shadowing

Finally, import those types into your executable with `use` statements, and
chainge `main`s function signature to return `Result<()>`. All functions in your
library that may fail will pass these `Results` back down the stack all the way
to `main`, and then to the Rust runtime, which will print an error.

Run `cargo check` to look for compiler errors, then fix them. For now it's
ok to end `main` with `panic!()` to make the project build.

_Set up your error handling strategy before continuing._

As with the previous project, you'll want to create placeholder data structures
and methods so that the tests compile. Now that you have defined an error type
this should be straightforward. Add panics anywhere necessary to get the test to
compile (`cargo test --no-run`).


## Part 2: Storing writes in a write-ahead log

Now we are finally going to begin implementing the beginnings of a real
database, by storing its contents to disk. You will use [`serde`] to serialize
the "get" command to a string, and the standard file I/O APIs to write it to
disk.

[`serde`]: https://serde.rs/

This is the basic behavior of `kvs` with a write-ahead log:

- "set"
  - The user invokes `kvs set mykey, myvalue`
  - `kvs` creates a struct representing the "get" command, containing its key and
    value
  - It then serializes that command to a `String`
  - It then appends the serialized command to a file containing the WAL
  - If that succeeds, it exits silently with error code 0
  - If it fails, it exits by printing the error and return a non-zero error code
- "get"
  - The user invokes `kvs get mykey`
  - `kvs` deserializes the entire WAL, one command at a time, executing the
    command against an in-memory map, and thus reconstructing the database state
    in memory
  - It then checks the map for the key
  - If it succeeds, it prints the key to stdout and exits with exit code 0
  - If it fails, it prints "Key not found", and exits with exit code 0

You will start by implementing the "set" flow. There are a number of steps here.
Most of them are straightforward to implement and you can verify you've done so
by running the appropriate `cli_*` test cases.

`serde` is a large library, with many options, and supporting many serialization
formats. Basic serialization and deserialization only requires annotating
your data structure correctly, and calling a function to write it
either to a `String` or a stream implementing `Write`.

You need to pick a serialization format. Think about the properties you want in
your serialization format &mdash; do you want to prioritize performance? Do you
want to be able to read the content of the WAL in plain text? It's your choice,
but maybe you should include a comment in the code explaining it.

Some of the APIs you will call may fail, return a `Result` of some error type.
Make sure that your calling functions return a `Result` of _your own_ error type,
and that you convert between the two with `?`.

You may implement the "set" command now, or you can proceed to the next section
to read about the "get" command. It may help to keep both in mind, or to
implement them both simultaniously. Your choice.


## Part 3: Reading from the write-ahead log




<!--

## TODOs

- is there a term for converting a WAL to it's permanent format?
- custom main error handling
- limits on k/v size?
- maintaining data integrity on failure 
- todo: `Result<Option<String>>` vs `Result<String>`
  - is "not found" exit code 0 or !0?
- error context
- serialize directly to file stream

-->
