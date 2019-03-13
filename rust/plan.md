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
- (Optionally) Submit project for online grading via [TODO].




## Section 1 (Setup)

- [Project: Tools and good bones][p-tools]

  - Task: create an in-memory key/value store that passes simple tests
    and responds to command-line arguments.

  - Goals:
    - Get a toolchain and tools
    - Use `cargo init / run / test`
    - Use external crates
    - Define a data type for a key-value store

  - Topics: clap, testing, CARGO_VERSION, clippy, rustfmt

  - Extensions: structopt, log / slog

- [Lesson: Whirlwind Rust][t-whirlwind] ([slides][s-whirlwind])

  - Readings:
    - [The Book - Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
    - [The Book - Programming a Guessing Game](https://doc.rust-lang.org/book/ch02-00-guessing-game-tutorial.html)
    - [The Book - Common Programming Concepts](https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html)

  - Topics: Rust and cargo, ownership & borrowing / aliasing & mutability in
    brief, resources (TODO: what else?)

- Lesson: Data structures in Rust

  - Topics: when to use which struct types, impls, ctor patterns, dtors, reprs,
    padding demo, packed structs, size and alignment in depth, enum
    implementation and optimizations,

- Lesson: Crates and crates.io

  - Topics: importing crates, features, debugging and fixing dependencies,
    std vs crate philosophy and history, finding crates

- Lesson: Rust tooling

  - Topics: `#[test]`, how does test work under the hood?, what does 'cargo run'
    actually do?, clippy, rustfmt, controlling clippy and rustfmt, links to
    other useful tools, cargo / rustc wrapping pattern in depth (ex rustup,
    RUSTC_WRAPPER)

- Lesson: Formatting, println et. al, log, and slog

  - Readings:
    - [The Book - Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
    - [`std::fmt`](https://doc.rust-lang.org/std/fmt/index.html)

  - Topics: formatting tips, derive Debug in depth, how does format! work?, log
    demo, slog and structured logging, env_logger, APIs that take format
    strings,

  - TODO: This subject isn't _needed_ to do the project, but formatting
    is a good deep-dive opportunity





## Section 2 (File I/O)

- Project: File I/O

  - Task: create a persistent key/value store that can be accessed from the
    command line

  - Goals:
    - Handle and report errors robustly
    - Accept command line arguments
    - Write data to disk as a write-ahead log
    - Read the state of the key/value store from disk
    - Use serde for serialization
    - Maintain consistency after crashes or data corruption?
    - Use file I/O APIs to binary search

  - Extensions: range queries, convert full WAL to binary-searchable file

- Lesson: Proper error handling

  - Readings:
    - [The Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
    - [Rust Error Handling](http://blog.burntsushi.net/rust-error-handling/)
    - [The `failure` book](https://rust-lang-nursery.github.io/failure/)

  - Topics: TODO, `fn main() -> Result`

- Lesson: Collections and iterators




## Section 3 (Networking)

- Project: Networking

  - Task: create a single-threaded, persistent key/value store server and client
    with synchronous networking over the HTTP and GRPC protocols.

- Lesson: Basic network APIs

  - Topics: std networking, TCP vs UDP?, reqwest, blocking HTTP serving w/ Iron/Hyper?

- Lesson: Build-time Rust

  - Topics: build scripts, protobuf compilation example, getting rustc version
    and other useful info (TODO crates for this?), in-depth examples of crates
    that rely on build scripts (e.g. skeptic, TODO), (TODO what else?)

- Lesson: GPRC and prost




## Section 4 (Parallelism)

- Project: Parallelism

  - Task: create a multi-threaded, persistent key/value store server and client
    with synchronous networking via the GRPC protocol.

  - Goals:
    - TODO
    - Benchmark single-threaded vs multi-threaded

- Lesson: The big problem &mdash; aliasing and mutability

  - Readings:
    - [The Book - Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
    - [The Nomicon - Aliasing](https://doc.rust-lang.org/nomicon/aliasing.html)
    - [The Problem with Single-threaded Shared Mutability](https://manishearth.github.io/blog/2015/05/17/the-problem-with-shared-mutability)

  - Topics: mutable aliasing bugs, how ownership prevents mutable aliasing, Sync
    / Sync, uniq / shared vs immutable / mutable, Rc and Arc, interior
    mutability in depth,

- Lesson: Ownership and borrowing in practice

  - Readings:
    - [Rust - A Unique Perspective](https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html)
    - [Too Many Linked Lists](https://cglab.ca/~abeinges/blah/too-many-lists/book)

  - Topics: when to use pass-by-value, the performance impact of moves,
    reference-bearing structs, (TODO: what other ownership and borrowing
    topics?)

- Lesson: Parallel Rust

  - Topics: sharing vs message passing, thread pools, (TODO what else?)

- Lesson: Benchmarking, profiling, and debugging

  - Topics: println debugging, RUST_BACKTRACE, perf, callgrind?, bpftrace?, criterion and critcmp, gdb and
    Rust, blocking/ (http://www.brendangregg.com/offcpuanalysis.html), (TODO which
    profiling / bloat tools?), std bench vs criterion, black_box and
    alternatives, how does rust benchmarking work?,




## Section 5 (Async)

- Project: Async I/O

  - Task: create a multi-threaded, persistent key/value store server and client
    with asynchronous networking via the GRPC protocol.

  - Goals:
    - Define and compile a GRPC protocol
    - Use tokio, prost and tower-grpc for networking
    - Support range queries
    - Understand the distinction between concurrency and parallelism
    - Use a thread pool to prevent "blocking"

  - Extensions: tokio-file (or whatever), crash recovery, async/await, LSM
    compaction

- Lesson: Basic futures

  - Readings
    - [The What and How of Futures and async/await in Rust](https://www.youtube.com/watch?v=9_3krAQtD2k)

  - Topics: what are futures?, how to think in futures, futures patterns,
    mio? (probably should go somewhere before tokio)

- Lesson: Async I/O with Tokio and Tower

  - Readings:
    - [The Tokio docs](https://tokio.rs/docs/) - todo maybe list specific sections

  - Topics: (TODO need to research further), alternatives to tokio

- Lesson: `async` / `await`

  - Topics: futures vs async/await, how does async / await work,
    async borrowing, Pin




<!-- lesson and project links -->


<!-- section 1 -->

[p-tools]: projects/tools/project.md
[t-whirlwind]: lessons/whirlwind.md
[s-whirlwind]: lessons/whirlwind.slides.html

