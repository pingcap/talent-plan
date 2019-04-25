# Project: Tools and good bones

**Task**: Create an in-memory key/value store that passes simple tests and responds
to command-line arguments.

**Goals**:

- Install the Rust compiler and tools
- Learn the project structure used throughout this course
- Use `cargo init` / `run` / `test` / `clippy` / `fmt`
- Learn how to find and import crates from crates.io
- Define an appropriate data type for a key-value store

**Topics**: testing, clap, `CARGO_VERSION` etc., clippy, rustfmt

**Extensions**: structopt


## Introduction

In this project you will create a simple in-memory key/value store that passes
some tests and responds to command line arguments. The focus of this project is
on the tooling and setup that goes into a typical Rust project.


## Project spec

The cargo project, `kvs`, builds a command-line key-value store client called
`kvs`, which in turn calls into a library called `kvs`.

The `kvs` executable supports the following command line arguments:

- `kvs set <KEY> <VALUE>`

  Set the value of a string key to a string

- `kvs get <KEY>`

  Get the string value of a given string key

- `kvs rm <KEY>`

  Remove a given key

- `kvs -V`

  Print the version

The `kvs` library contains a type, `KvStore`, that supports the following
methods:

- `KvStore::set(key: String, value: String)`

  Set the value of a string key to a string

- `KvStore::get(key: String) -> Option<String>`

  Get the string value of the a string key. If the key does not exist,
  return `None`.

- `KvStore::remove(key: String)`

  Remove a given key.

The `KvStore` type stores values in-memory, and thus the command-line client can
do little more than print the version. The `get`/ `set` / `rm` commands will 
return an "unimplemented" error when run from the command line. Future projects 
will store values on disk and have a working command line interface.


## How to treat these projects

The more you find the answers for yourself, the more you learn. So even though
this course comes with solutions and answers to every project, don't just go
read the answers and move on. Read the material, do the projects, and answer the
questions yourself.

It is ok to get help though. In particular, you are encouraged
to ask questions an the #beginners channel in [the Rust Discord][rd]
or the #rust-beginners channel or [Mozilla IRC][mi].

[rd]: https://discord.gg/rust-lang
[mi]: https://chat.mibbit.com/?server=irc.mozilla.org&channel=%23rust

For each project, your task is to make the provided tests pass, as described
below. If you are submitting the project for evaluation, then this is the part
that will be evaluated. In addition, the projects pose questions periodically.
These are to encourage you to think deeper about the subject matter, though your
answers won't be evaluated.


## Installation

At this point in your Rust programming experience you should know
how to install Rust via [rustup].

[rustup]: https://www.rustup.rs

If you haven't already, do so now by running

```
curl https://sh.rustup.rs -sSf | sh
```

(If you are running Windows then follow the instructions on rustup.rs. Note
though that you will face more challenges than others during this course, as it
was developed on Unix. In general, Rust development on Windows is as less
polished experience than on Unix).

Verify that the toolchain works by typing `rustc -V`. If that doesn't work, log
out and log in again so that changes to the login profile made during
installation can take effect.


## Project setup

You will do the work for this project in your own git repository, with your own
Cargo project. You will import the test cases for the project from the [source
repository for this course][course].

[course]: https://github.com/pingcap/talent-plan

Note that within that repository, all content related to this course is within
the `rust` subdirectory. You may ignore any other directories.

The projects in this course contain both libraries and executables. They are
executables because we are developing an application that can be run. They are
libraries because the supplied test cases must link to them.

We'll use the same setup for each project in this course.

The directory layout we will use is:

```
├── Cargo.toml
├── src
│   ├── bin
│   │   └── kvs.rs
│   └── lib.rs
└── tests
    └── tests.rs
```

The `Cargo.toml`, `lib.rs` and `kvs.rs` files look as follows:

`Cargo.toml`:

```toml
[package]
name = "kvs"
version = "0.1.0"
authors = ["Brian Anderson <andersrb@gmail.com>"]
description = "A key-value store"
edition = "2018"
```

`lib.rs`:

```rust
// just leave it empty for now
```

`kvs.rs`:

```rust
fn main() {
    println!("Hello, world!");
}
```

The `name` and `authors` values can be whatever you like, and the author should
be yourself. Note though that the contents of `kvs.rs` are affected by the
package name, which is also the name of the library within the package. (TODO
clarify)

Finally, the `tests` directory is copied from the course materials. In this case,
copy from the course repository the file `rust/project/tools/tests`
into your own repository, as `tests`.

You may set up this project with `cargo new --lib`, `cargo init --lib`, or
manually. You'll probably also want to initialize a git repository in the same
directory.

At this point you should be able to run the program with `cargo run`. It should 

_Try it now._

You are set up for this project and ready to start hacking.


## Part 1: Make the tests compile

You've been provided with a suite of unit tests in `tests/tests.rs`. Open it up
and take a look.

Try to run the tests with `cargo test`. What happens? Why?

Your first task for this project is to make the tests _compile_. In `src/lib.rs`
write the type and method definitions necessary to make `cargo test --no-run`
complete successfully. Don't write any method bodies yet &mdash;
instead write `panic!()`.

_Do that now before moving on._

Once that is done, if you run `cargo test` (without `--no-run`),
you should see that some of your tests are failing, like

```
TODO insert after we have a sample project
```

In practice, particularly with large projects, you won't run the entire set of
test suites while developing a single feature. To narrow down the set of tests
to the ones we care about, run the following:

```
cargo test --test tests
```

That reads pretty silly: "test test tests". What it means though is that we're
testing (`cargo test`), we want to run a specific suite of tests (`--test`), in
this case the tests in `tests.rs` (`--test tests`). We can even narrow our testing
down to a single test case within the `tests` test suite, like

```
cargo test --test tests -- cli_no_args
```

or, by matching multiple test cases, a few, like:


```
cargo test --test tests -- cli
```

_Try it now._

That's probably how you will be running the tests yourself as you work
through the project, otherwise you will be distracted by the many failing tests
that you have not yet fixed.


## Part 2: Accept command line arguments

The key / value stores throughout this course are all controlled through a
command-line client. In this project the command-line client is very simple
because the state of the key-value store is only stored in memory, not persisted
to disk.

In this part you will make the `cli_*` test cases pass.

Recall how to run individual test cases from previous sections
of 

Again, the interface for the CLI is:

- `kvs set <KEY> <VALUE>`

  Set the value of a string key to a string

- `kvs get <KEY>`

  Get the string value of a given string key

- `kvs rm <KEY>`

  Remove a given key

- `kvs -V`

  Print the version

In this iteration though, the `get` and `set` commands will print to stderr the
string, "unimplemented", and exiting with a non-zero exit code, indicating an
error.

You will use the `clap` crate to handle command-line arguments.

<i>Use [crates.io](https://crates.io) to find the documentation
for the `clap` crate, and implement the command line interface
such that the `cli_*` test cases pass.</i>

When you are testing, use `cargo run`; do not run the executable directly from
the `target/` directory. When passing arguments to the program, separate them
from the `cargo run` command with two dashes, `--`, like `cargo run -- get
key1`.


## Part 3: Cargo environment variables

When you set up `clap` to parse your command line arguments, you probably set
the name, version, authors, and description (if not, do so). This information is
redundant w/ values provided in `Cargo.toml`. Cargo sets environment variables
that can be accessed through Rust source code, at build time.

_Modify your clap setup to set these values from standard cargo environment
variables._




## Part 4: Store values in memory

Now that your command line scaffolding is done, let's turn to the implementation
of `KvStore`, and make the remaining test cases pass.

The behavior of `KvStore`'s methods are fully-defined through the test cases
themselves &mdash; you don't need any further description to complete the
code for this project.

_Make the remaining test cases pass by implementing methods on `KvStore`._


## Part 5: Documentation

You have implemented the project's functionality, but there are still a few more
things to do before it is a polished piece of Rust software, ready for
contributions or publication.

First, public items should generally have doc comments.

Doc comments are displayed in a crate's API documentation. API documentation can
be generated with the command, `cargo doc`, which will render them as HTML to
the `target/doc` folder. Note though that `target/doc` folder does not contain
an `index.html`. Your crate's documentation will be located at
`target/doc/kvs/index.html`. You can launch a web browser at that location with
`cargo doc --open`.

[Good doc comments][gdc] do not just repeat the name of the function, nor repeat
information gathered from the type signature. They explain why and how one would
use a function, what the return value is on both success and failure, error and
panic conditions. The library you have written is very simple so the
documentation can be simple as well. If you truly cannot think of anything
useful to add through doc comments then it can be ok to not add a doc comment
(this is a matter of preference). With no doc comments it should be obvious how
the type or function is used from the name and type signature alone.

Doc comments contain examples, and those examples can be tested with `cargo test
--doc`.

You may want to add `#![deny(missing_docs)]` to the top of `src/lib.rs`
to enforce that all public items have doc comments.

_Add doc comments to the types and methods in your library. Follow the
[documentatine guidelines][gdc]. Give each an example and make sure they pass
`cargo test --doc`._

[gdc]: https://rust-lang-nursery.github.io/api-guidelines/documentation.html


## Part 6: Ensure good style with `clippy` and `rustfmt`

`clippy` and `rustfmt` are tools for enforcing common Rust style. `clippy` helps
ensure that code uses modern idioms, and prevents patterns that commonly lead to
errors. `rustfmt` enforces that code is formatted consistently.

Both tools are included in the Rust toolchain, but not installed by default.
They can be installed with the following commands:

```
rustup component add clippy
rustup component add rustfmt
```

_Do that now._

Both tools are invoked as cargo subcommands, `clippy` as `cargo clippy` and
`rustfmt` as `cargo fmt`. Note that `cargo fmt` modifies your source code, so
you might want to check in any changes before running it to avoid accidentally
making unwanted changes, after which you can include the changes as part of the
previous commit with `git commit --amend`.

_Run `clippy` against your project and make any suggested changes. Run `rustfmt`
against yur project and commit any changes it makes._

Congratulations, you are done with project 1! If you like you
may complete the remaining "extensions". They are optional.


## Extension 1: `structopt`

In this project we used `clap` to parse command line arguments. It's typical to
represent a program's parsed command line arguments as a struct, perhaps named
`Config` or `Options`. Doing so requires calling the appropriate methods on
`clap`'s `ArgMatches` type. Both steps, for larger programs, require _a lot_ of
boilerplate code. The `structopt` crate greatly reduces boilerplate by allowing
you to define a `Config` struct, annotated to automatically produce a `clap`
command line parser that produces that struct. Some find this approach nicer
than writing the `clap` code explicitly.

_Modify your program to use `structopt` for parsing command line
arguments instead of using `clap` directly._


<!--

## TODOs

- use "test suite" as-is here or pick different terminology?
- set the binary's name
- ask about pros / cons of this main.rs setup
  - explain why we're doing this setup
    (makes main testable) though this will
	become evident as they work through the tests
- doc comments
- make sure there's enough background reading to support the project
- resources (whether / where to put these?)
  - https://docs.rs/clap/2.32.0/clap/
  - https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo
  - https://rust-lang-nursery.github.io/api-guidelines/documentation.html#documentation
  - https://doc.rust-lang.org/std/macro.env.html
  - https://github.com/rust-lang/rust-clippy/blob/master/README.md
  - https://github.com/rust-lang/rustfmt/blob/master/README.md
- do range lookups (`scan`)?
- README.md?
- GitHub CI setup?

-->
