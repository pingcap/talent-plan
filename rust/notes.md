# Implementors' notes

## Desired subjects

This is a more full list of topics to cover than
in the README.

- error handling
  - simple vs. complex, Fail vs StdError, etc
  - `fn main() -> Result`
  - `panic!` and unwinding
- logging w/ log and slog
  - how env_logger works?
- trees vs maps
- async vs sync networking
  - `std` networking
  - TCP vs UDP
  - `reqwest`
  - blocking HTTP serving w/ Iron
- sync file io and solutions to blocking
- buffered vs unbuffered i/o
- benchmarking, criterion and critcmp, black_box
- RUST_BACKTRACE
- where to ask questions
- futures
- tokio
- mio?
- async/await? - maybe future iterations
- Pin?
- generic placeholder idiom - let foo: Vec<_> =
- construction w/ iterators idiom
- semver trick
- impl trait, the Into<Option<_>> trick
- Rust history, culture, design principles
- rustfmt, clippy, configuring both
- rust 2018?
  - we'll just asume rust 2018 and not mention 2015 unless necessary
- tools most rust programmers should know
- build scripts
  - protobuf compilation example
  - getting rustc version
  - in-depth examples of crates that rely on build scripts
- using RUSTFLAGS
- debugging
- profiling
- how not to write Rust (bad practices from other languages)
  - Ana likes this
- variable shadowing
- DSTs
- configuring clippy / rustfmt
- scripting clippy / rustfmt for CI
- CI setup
- when to use which struct types, impls, ctor patterns, dtors, reprs,
  padding demo, packed structs, size and alignment in depth, enum
  implementation and optimizations
- importing crates, features, debugging and fixing dependencies,
  std vs crate philosophy and history, finding crates
- how does testing work?
- what does `cargo run` do?
- cargo / rustc wrapping pattern in depth (ex rustup, `RUSTC_WRAPPER`)
- formatting tips, derive Debug in depth, how does `format!` work?
- mutable aliasing bugs, how ownership prevents mutable aliasing
- Sync / Send, uniq / shared vs immutable / mutable
- `Rc` and `Arc`, interior mutability in depth
- when to use pass-by-value, the performance impact of moves
- reference-bearing structs
- sharing vs message passing
- thread pools

## Sources

- https://pdos.csail.mit.edu/6.824/schedule.html
  - The MIT distributed systems course this course
    is inspired by and intended to precede
- https://github.com/ferrous-sytems/rust-three-days-course
- https://github.com/nrc/talks
- https://docs.google.com/document/d/11P5f5VRKhS7ZOB5_sbnCKJLo_zor9BiwEAZ9GvVyHWE/edit#
  - previous brainstorming
- RustBridge

## Readings

- http://highscalability.com/blog/2011/1/10/riaks-bitcask-a-log-structured-hash-table-for-fast-keyvalue.html

## Subjects to potentially cut

- Parallelism section
- Formatting lesson
- Build time lesson
- Collections and iterators

## Grading

- automated text-based cheat detection
- automated grading of test cases
- automated grading of non-unit-test requirements via python script
- how to grade freeform answers?

## TODO

- do survey of other sources' subject progression
- lessons and labs pose questions
- have a chinese native identify and remove 'hard' words and phrases
- mention somewhere that we're using rust 2018 only, how to verify
- change slides.html URLS to link to the hosted file
- add more description of how to treat the projects
  - note that improvements beyond the scope of the project are encouraged - comment them
- add TOC to each project
- use fail-rs for consistency tests
- mention the test names that should be focused on each part of each project
- find a way to use specialization via the KvsEngine trait
