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
- workspaces
- refactoring
- https://github.com/altsysrq/proptest
- fuzzing
- dbg!
- shadowing

## Sources

- https://pdos.csail.mit.edu/6.824/schedule.html
  - The MIT distributed systems course this course
    is inspired by and intended to precede
- https://github.com/ferrous-sytems/rust-three-days-course
- https://github.com/nrc/talks
- RustBridge

## Pedagogy

- https://launchschool.com/pedagogy
- https://launchschool.com/is_this_for_me

## Reading sources

- previous reading ideas https://github.com/pingcap/talent-plan/blob/32311e6999a2a5b7db25cd2b4dd96491d5181165/rust/plan.md
- http://highscalability.com/blog/2011/1/10/riaks-bitcask-a-log-structured-hash-table-for-fast-keyvalue.html
- https://github.com/brson/rust-anthology/blob/master/master-list.md
- https://github.com/ctjhoa/rust-learning
- https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
- https://rust-lang-nursery.github.io/failure/
- https://serde.rs/
- https://doc.rust-lang.org/cargo/
- https://medium.com/rabiprasadpadhy/google-spanner-a-newsql-journey-or-beginning-of-the-end-of-the-nosql-era-3785be8e5c38
- https://github.com/Hexilee/async-io-demo
- https://stjepang.github.io/2019/01/29/lock-free-rust-crossbeam-in-2019.html
- https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html
- https://doc.rust-lang.org/nomicon/aliasing.html
- https://manishearth.github.io/blog/2015/05/17/the-problem-with-shared-mutability/
- https://www.youtube.com/watch?v=9_3krAQtD2k (futures async await)
- https://shipilev.net/blog/2014/java-scala-divided-we-fail/
- https://www.internalpointers.com/post/lock-free-multithreading-atomic-operations

## Exercise sources

- https://github.com/rust-lang/rustlings
- https://exercism.io/tracks/rust
- https://doc.rust-lang.org/rust-by-example/index.html

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
- use imperative "part" titles? "creating" vs "create"
- add explanation of "blocking" to parallelism / thread pool project
- per readme, make parallelism project "fun and foolproof"
- per readme, incorporate critcmp
- deleted from readme:
  - Build scripts and the interaction between build-time and run-time
  - Insights into inner workings of the language and libraries
  - to look "under the hood" and understand how and why Rust works like it does
  - the course can be delivered offline
  - _lessons_ - focused coverage of subjects required for writing practical
    software of the type developed in the corresponding projects, including
    advanced tips, best practices, and deep dives. This are in the form of slides,
    speaker notes, and short _writeups_.
  - removed goal of teaching how to find information on your own about rust
    - it's now an implicit goal
- be clearer about crate deps
- consistent formatting for crate names (code or not?) (yes, to offset them as crates when not explicitly mentioned)
- do better at identifing and explaining the individual problems being solved
- add tips for finding documentation and crates
- mention mdbook when first linking to a book
- rustup doc and cargo doc and where both draw their content from
- make project contents more consistent
  - projects contain sections that are unrelated to the main subject
- change project.md to description.md, to get rid of the path stuttering
- replace wikipedia, docs.rs, official rust, readings with blog posts, papers
  and other more specific, harder to find resources
- create place to submit solutions
- compare criterion to bench
- move all links to ends of documents and explain in contributing.md
- sort lines of links
- how to find the names and descriptions of all lints
- consider adding links to project source code somewhere
- add note encouraging reading the rustfmt / rustclippy docs
- replace "_Note: ..." with "_**Note**: ..."
- describe in more depth the final product
- put a reminder about building blocks at top of every project
- add back-links in some non-intrusive way, somehow
- to p1 just add a list of useful Rust cli tools!
- sort deps in manifests
- after p1 link first uses of crate names to docs.rs
- say why we're doing each exercise or diversion
- publishing on crates.io
- critcmp

# Future projects

- 5 - use futures + http, use standard http bench tools
- 6 - use async/await
- 7 - use grpc
- 8 - focus on data integrity
- 9 - try to match performance of the production components
- ? - streaming scan operation with lifetimes and streaming network api

# survey

- time to complete
