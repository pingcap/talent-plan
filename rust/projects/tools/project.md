# Project: Tools and good bones

## Introduction

## How to treat these projects

TODO:

re answers to questions, solutions to problems, collaboration

## Setup

You will do the work for this project (and all projects in this course) in your
own git repository, with your own Cargo project. You will import the test
cases for the project from the [source repository for this course][course].

[course]: https://github.com/pingcap/talent-plan

Note that within that repository, all content related to this course is within
the `rust` subdirectory. You may ignore any other directories.

The projects in this course are both libraries and executables. They are
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

**Question A**: what does this directory layout suggest about how `lib.rs`,
`main.rs`, and `tests.rs` are compiled, and how they are linked to each other?
[Answers](answers.md#question-a).

The `Cargo.toml`, `lib.rs` and `main.rs` files look as follows:

`Cargo.toml`:

```toml
[package]
name = "mypackage"
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
    mypackage::main()
}
```

The `name` and `authors` values can be whatever you like, Note though that the
contents of `main.rs` are affected by the package name, which is also the name
of the library within the package.

Finally, the `tests.rs` file is copied from the course materials. In this case,
copy from the course repository the file `rust/project/tools/tests/tests.rs`
into your own repository, as `tests/tests.rs`

You may set up this project with `cargo new --lib`, `cargo init --lib`, or
manually. You'll probably also want to initialize a git repository in the same
directory.

**Question B**: this is the simplest project setup that accomplishes our goals. In
practice we might not name `src/bin/main.rs` as `main.rs`. Why not? What's the
name of our binary? What are two ways we could change the name of that binary?
Try it yourself.
[Answers](answers.md#question-b).

At this point you should be able to run the program with `cargo run`, and the
tests with `cargo test`. All the tests will fail, like

```
TODO
```

**Question C**: notice that there are _four_ different set of tests running (each could be
called a "test suite"). Where do each of those test suites come from?
[Answers](answers.md#question-c).

(TODO: use "test suite" like this or pick different terminology?)

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
cargo test --test tests -- TODO
```

And that's probably how you will be running the tests yourself as you work
through the project, otherwise you will be distracted by the many failing tests
that you are not currently working on fixing.

**Question D**: even if a given test suite doesn't contain any tests, why
might we not want them to run? Besides issuing the above command, how could
we permanently disable the three test suites we don't care about by
editing the project manifest (`Cargo.toml`)? [Answers](answers.md#question-d).

Now you are set up for this project and ready to start hacking.


## Part 1: TODO
