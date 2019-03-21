# Project: Tools and good bones

**Task**: Create an in-memory key/value store that passes simple tests and responds
to command-line arguments.

**Goals**:

- Install the Rust compiler and tools
- Learn the project structure used throughout this course
- Use `cargo init` / `run` / `test` / `clippy` / `fmt`
- Use external crates
- Define a data type for a key-value store

**Topics**: clap, testing, `CARGO_VERSION`, clippy, rustfmt

**Extensions**: structopt, log / slog


## Introduction

In this project you will create a simple in-memory key/value store that passes
some tests and responds to command line arguments. The focus of this project is
on the tooling and setup that goes into a typical Rust project.


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


## Project Setup

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
Cargo.toml
  src/
    lib.rs
	bin/
	  main.rs
  tests/
    tests.rs
```

**Question A**: What does this directory layout suggest about how `lib.rs`,
`main.rs`, and `tests.rs` are compiled, and how they are linked to each other?
How many libraries and executables will this project build? Which source files
are compiled into each library and executable? Which executables link to which
libraries?
[Answers](answers.md#question-a).

The `Cargo.toml`, `lib.rs` and `main.rs` files look as follows:

`Cargo.toml`:

```toml
[package]
name = "project_1"
version = "0.1.0"
authors = ["Brian Anderson <andersrb@gmail.com>"]
edition = "2018"
```

`lib.rs`:

```rust
pub fn main() {
    println!("Hello, world!");
}
```

`main.rs`:

```rust
fn main() {
    project_1::main()
}
```

The `name` and `authors` values can be whatever you like, Note though that the
contents of `main.rs` are affected by the package name, which is also the name
of the library within the package. (TODO clarify)

Finally, the `tests.rs` file is copied from the course materials. In this case,
copy from the course repository the file `rust/project/tools/tests/tests.rs`
into your own repository, as `tests/tests.rs`

You may set up this project with `cargo new --lib`, `cargo init --lib`, or
manually. You'll probably also want to initialize a git repository in the same
directory.

**Question B**: This is the simplest project setup that accomplishes our goals. In
practice we might not name `src/bin/main.rs` as `main.rs`. Why not? What's the
name of our binary? What are two ways we could change the name of that binary?
Try it yourself.
[Answers](answers.md#question-b).

At this point you should be able to run the program with `cargo run`.

Try it now.

You are set up for this project and ready to start hacking.


## Part 1: Make the tests compile

You've been provided with a suite of unit tests in `tests/tests.rs`. Open it up
and take a look.

Try to run the tests with `cargo test`. What happens? Why?

Your first task for this project is to make the tests _compile_. In `src/lib.rs`
write the type and method definitions necessary to make `cargo test --no-run`
complete successfully. Don't write any method bodies yet &mdash;
instead write `panic!()`.

Do that now before moving on.

Once that is done, if you run `cargo test` (without `--no-run`),
you should see that some of your tests are failing, like

```
TODO insert after we have a sample project
```

**Question C**: Notice that there are _four_ different set of tests running
(each could be called a "test suite"). Where do each of those test suites come
from? [Answers](answers.md#question-c).

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
cargo test --test tests -- (TODO after we have a test name to put here)
```

Try it.

That's probably how you will be running the tests yourself as you work
through the project, otherwise you will be distracted by the many failing tests
that you have not yet fixed.

**Question D**: even if a given test suite doesn't contain any tests, why
might we not want them to run? Besides issuing the above command, how could
we permanently disable the three test suites we don't care about by
editing the project manifest (`Cargo.toml`)? [Answers](answers.md#question-d).


## Part 2: Accept command line arguments


## TODOs

- use "test suite" as-is here or pick different terminology?
- set the binary's name
- ask about pros / cons of this main.rs setup
  - explain why we're doing this setup
    (makes main testable) though this will
	become evident as they work through the tests


