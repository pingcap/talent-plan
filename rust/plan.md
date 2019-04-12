# Practical Networked Applications in Rust - Lesson plan

This course is divided into five _sections_, each with a _project_ designed to
give you exposure to practical Rust programming subjects.

Each project builds on experience from the previous project. It is reasonable to
start each project by copy-pasting the previous.

As you work through the course content, _please_ be on the lookout for things
you would improve about the course, and either [submit issues][si] explaining,
or [submit pull requests][spr] with improvements. _Each accepted pull request
counts toward extra credit during final evaluation_. Pull requests to any other
project used during this course count as well. This is an opportunity to gain
experience contributing to an open-source Rust project. Make this a better
course for the next student to take it than it was for you. That is how open
source projects evolve.

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

## Section 3 (Networking)


### [Project: Synchronous client-server networking][p-net]

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

## Section 4 (Parallelism)


### [Project: Parallelism, profiling, and benchmarking][p-par]

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread-pool
- Use crossbeam channels
- Perform compaction in a background thread
- Benchmark single-threaded vs multi-threaded

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
