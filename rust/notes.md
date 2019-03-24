# Implementors' notes

## Desired subjects

This is a more full list of topics to cover than
in the README.

- error handling
  - simple vs. complex, Fail vs StdError, etc
- logging w/ log and slog
  - how env_logger works?
- trees vs maps
- async vs sync networking
- sync file io and solutions to blocking
- buffered vs unbuffered i/o
- benchmarking, criterion and critcmp, black_box
- RUST_BACKTRACE
- how to find crates
- where to ask questions
- futures
- tokio
- async/await? - maybe future iterations
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

## Sources

- https://pdos.csail.mit.edu/6.824/schedule.html
  - The MIT distributed systems course this course
    is inspired by and intended to precede
- https://github.com/ferrous-sytems/rust-three-days-course
- https://github.com/nrc/talks
- https://docs.google.com/document/d/11P5f5VRKhS7ZOB5_sbnCKJLo_zor9BiwEAZ9GvVyHWE/edit#
  - previous brainstorming
- RustBridge

## Subjects to potentially cut

- Whirlwind rust lesson
- Parallelism section
- Formatting lesson
- Build time lesson
- Collections and iterators

## TODO

- do survey of other sources' subject progression
- lessons and labs pose questions
- have a chinese native identify and remove 'hard' words and phrases
- mention somewhere that we're using rust 2018 only, how to verify
- change slides.html URLS to link to the hosted file
