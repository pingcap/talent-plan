# Practical Networked Applications in Rust - Lesson plan

This course is divided into five _sections_, each with a _project_ designed to
give you exposure to practical Rust programming subjects. Each section's
_lessons_ and accompanying external pre-requisite _readings_ provide the
resources necessary to complete the projects.

Note that it is not sufficient to follow the lessons without reading the
external reading material. The readings are the foundation necessary to
understand and complete the projects; the lessons themselves are intended to
provide additional insight and in-depth explorations beyond the background
reading.

Each project builds on experience from the previous project. It is reasonable to
start each project by copy-pasting the previous.

A suggested workflow:

- Skim the project for the next section to get an idea of what you need to learn
  from the next lessons.
- Prior to following a lesson, read the pre-requisite readings.
- During the lesson, follow the slides, and
  - if taking the course live, ask questions
  - if taking the course online, read the writeups that accompany the slides
- Follow each project according to their own instructions, writing Rust programs
  that pass the projects' accompanying tests.

As you work through the course content, _please_ be on the lookout for things
you would improve about the course, and either [submit issues][si] explaining,
or [submit pull requests][spr] with improvements. _Each accepted pull request
counts toward extra credit during final evaluation_. Pull requests to any other
project used during this course count as well. This is an opportunity to gain
experience contributing to an open-source Rust project. Make this a better
course for the next to take it than it was for you. That is how open source
projects evolve.

[si]: https://github.com/pingcap/talent-plan/issues/new
[spr]: https://github.com/pingcap/talent-plan/compare




## Section 1 (Setup)


### [Project: Tools and good bones][p-tools]

**Task**: Create an in-memory key/value store that passes simple tests and responds
to command-line arguments.

**Goals**:

- Install the Rust compiler and tools
- Learn the project structure used throughout this course
- Use `cargo build` / `run` / `test` / `clippy` / `fmt`
- Use external crates
- Define a data type for a key-value store

**Topics**: clap, testing, `CARGO_VERSION`, clippy, rustfmt

**Extensions**: `structopt`, `log` / `slog`,


### [Lesson: Data structures in Rust][t-data] ([slides][s-data])

**Topics**: when to use which struct types, impls, ctor patterns, dtors, reprs,
padding demo, packed structs, size and alignment in depth, enum
implementation and optimizations,


### [Lesson: Crates and crates.io][t-crates] ([slides][s-crates])

**Topics**: importing crates, features, debugging and fixing dependencies,
std vs crate philosophy and history, finding crates


### [Lesson: Rust tooling][t-tools] ([slides][s-tools])

**Readings**:
  - [`clippy` README](https://github.com/rust-lang/rust-clippy/blob/master/README.md)
  - [`rustfmt` README](https://github.com/rust-lang/rustfmt/blob/master/README.md)

**Topics**: `#[test]`, how does test work?, what does `cargo run` actually do?,
clippy, rustfmt, controlling clippy and rustfmt, links to other useful tools,
cargo / rustc wrapping pattern in depth (ex rustup, `RUSTC_WRAPPER`)


### [Lesson: Formatting, println et. al, log, and slog][t-fmt] ([slides][s-fmt])

**Readings**:
 - [The Book - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
 - [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html)

**Topics**: formatting tips, derive Debug in depth, how does `format!` work?,
`log` demo, `slog` and structured logging, `env_logger`, APIs that take format
strings




## Section 2 (File I/O)


### [Project: File I/O][p-fs]

**Task**: Create a persistent key/value store that can be accessed from the
command line

**Goals**:

- Handle and report errors robustly
- Write data to disk as a write-ahead log
- Read the state of the key/value store from disk
- Use serde for serialization
- Use file I/O APIs to binary search

**Topics**: `failure` crate, `std::net::fs`, `Read` / `Write` traits,
serde

**Extensions**: range queries, store data using bitcast algo?

### [Lesson: Proper error handling][t-errors] ([slides][s-errors])

**Readings**:

- [The Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Rust Error Handling](http://blog.burntsushi.net/rust-error-handling/)
- [The `failure` book](https://rust-lang-nursery.github.io/failure/)

**Topics**: error patterns, `failure` crate, `fn main() -> Result`, `panic!` and
unwinding

### [Lesson: Collections and iterators][t-coll] ([slides][s-coll])




## Section 3 (Networking)


### [Project: Networking][p-net]

**Task**: Create a single-threaded, persistent key/value store server and client
with synchronous networking over the HTTP protocol.

**Goals**:

- Use hyper for synchronous networking


### [Lesson: Basic network APIs][t-net] ([slides][s-net])

**Topics**: `std` networking, TCP vs UDP, `reqwest`, blocking HTTP serving w/ Iron


### [Lesson: Build-time Rust][t-build] ([slides][s-build])

**Topics**: build scripts, protobuf compilation example, getting rustc version
and other useful info, in-depth examples of crates that rely on build scripts


### [Lesson: GPRC and prost][t-grpc] ([slides][s-grpc])




## Section 4 (Parallelism)


### [Project: Parallelism][p-par]

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking via HTTP.

**Goals**:

- Write a simple thread-pool
- Use crossbeam channels
- Benchmark single-threaded vs multi-threaded


### [Lesson: The big problem &mdash; aliasing and mutability][t-alias] ([slides][s-alias])

**Readings**:

- [The Book - Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [The Nomicon - Aliasing](https://doc.rust-lang.org/nomicon/aliasing.html)
- [The Problem with Single-threaded Shared Mutability](https://manishearth.github.io/blog/2015/05/17/the-problem-with-shared-mutability)

**Topics**: mutable aliasing bugs, how ownership prevents mutable aliasing, Sync
/ Sync, uniq / shared vs immutable / mutable, `Rc` and `Arc`, interior
mutability in depth,


### [Lesson: Ownership and borrowing in practice][t-own] ([slides][s-own])

**Readings**:

- [Rust - A Unique Perspective](https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html)
- [Too Many Linked Lists](https://cglab.ca/~abeinges/blah/too-many-lists/book)

**Topics**: when to use pass-by-value, the performance impact of moves,
reference-bearing structs


### [Lesson: Parallel Rust][t-par] ([slides][s-par])

**Topics**: sharing vs message passing, thread pools


### [Lesson: Benchmarking, profiling, and debugging][t-prof] ([slides][s-prof])

**Topics**: println debugging, RUST_BACKTRACE, perf, gdb and Rust, std bench vs
criterion, black_box and alternatives, how does rust benchmarking work?, other
tools




## Section 5 (Async)


### [Project: Async I/O][p-async]

**Task**: Create a multi-threaded, persistent key/value store server and client
with asynchronous networking via HTTP.

**Goals**:

- Use async hyper and futures
- Support range queries
- Understand the distinction between concurrency and parallelism
- Use a thread pool to prevent "blocking"

**Extensions**: crash recovery, async/await


### [Lesson: Basic futures][t-fut] ([slides][s-fut])

**Readings**:

- [The What and How of Futures and async/await in Rust](https://www.youtube.com/watch?v=9_3krAQtD2k)

**Topics**: what are futures?, how to think in futures, futures patterns, mio


### [Lesson: `async` / `await`][t-async-await] ([slides][s-async-await])

**Topics**: futures vs async/await, how does async / await work,
async borrowing, Pin




<!--

## TODOs

- reduce scope
- fmt subject isn't _necessary_ but is a deep-dive topic
- need to have a "how to get help" section somewhere

-->




<!-- lesson and project links -->


<!-- section 1 -->

[p-tools]: projects/tools/project.md
[t-whirlwind]: lessons/whirlwind.md
[s-whirlwind]: lessons/whirlwind.slides.html
[t-data]: lessons/data-structures.md
[s-data]: lessons/data-structures.slides.html
[t-crates]: lessons/crates.md
[s-crates]: lessons/crates.slides.html
[t-tools]: lessons/tools.md
[s-tools]: lessons/tools.slides.html
[t-fmt]: lessons/formatting.md
[s-fmt]: lessons/formatting.slides.html

<!-- section 2 -->

[p-fs]: projects/file-io/project.md
[t-errors]: lessons/error-handling.md
[s-errors]: lessons/error-handling.slides.html
[t-coll]: lessons/collections-and-iterators.md
[s-coll]: lessons/collections-and-iterators.slides.html

<!-- section 3 -->

[p-net]: projects/networking/project.md
[t-net]: lessons/networking.md
[s-net]: lessons/networking.slides.html
[t-build]: lessons/build-time.md
[s-build]: lessons/build-time.slides.html
[t-grpc]: lessons/grpc.md
[s-grpc]: lessons/gprc.slides.html

<!-- section 4 -->

[p-par]: projects/parallelism/project.md
[t-alias]: lessons/aliasing-and-mutability.md
[s-alias]: lessons/aliasing-and-mutability.slides.html
[t-own]: lessons/ownership-and-borrowing.md
[s-own]: lessons/ownership-and-borrowing.slides.html
[t-par]: lessons/parallelism.md
[s-par]: lessons/parallelism.slides.html
[t-prof]: lessons/profiling.md
[s-prof]: lessons/profiling.slides.html

<!-- section 5 -->

[p-async]: projects/async-io/project.md
[t-fut]: lessons/futures.md
[s-fut]: lessons/futures.slides.html
[t-async-await]: lessons/async-await.md
[s-async-await]: lessons/async-await.slides.html
