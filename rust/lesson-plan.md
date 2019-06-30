# PNA Rust Lesson plan

A training course about practical systems software construction in Rust.

Over a series of projects, you will build a networked, persistent [key-value
database][kv] with multithreading and asynchronous I/O. In between projects you
will study and practice individual subjects necessary to complete the project.
Along the way you will explore multiple designs and their tradeoffs.

See [README.md] for the overview, goals, audience, and prerequisites.

- [Prerequisites](#user-content-prerequisites)
- [Getting the materials](#user-content-getting-the-materials)
- [Course structure](#user-content-course-structure)
- [Getting help](#user-content-getting-help)
- [Making PNA Rust Better](#user-content-making-pna-rust-better)
- [Practical Networked Applications in Rust](#user-content-practical-networked-applications-in-rust)
  - [Building Blocks 1](#user-content-building-blocks-1)
  - [Project 1: The Rust toolbox and strong foundations](#user-content-project-1-the-rust-toolbox-and-strong-foundations)
  - [Building Blocks 2](#user-content-building-blocks-2)
  - [Project 2: Log-structured file I/O](#user-content-project-2-log-structured-file-io)
  - [Building Blocks 3](#user-content-building-blocks-3)
  - [Project 3: Synchronous client-server networking](#user-content-project-3-synchronous-client-server-networking)
  - [Building Blocks 4](#user-content-building-blocks-4)
  - [Project 4: Parallelism](#user-content-project-4-parallelism)
  - [Building Blocks 5](#user-content-building-blocks-5)
  - [Project 5: Asynchronous programming in Rust](#user-content-project-5-asynchronous-programming-in-rust)


## Prerequisites

As [described in the README.md][pre], this is not a course for novice
programmers, and there are significant prerequisites. Ensure that you meet them
all before proceeding.


## Getting the materials

All material for this course is in the

> https://github.com/pingcap/talent-training

git repository on GitHub, in the [`rust` subdirectory][rs]. You will want a copy
of it on your local computer, particularly for easy access to the conformance
tests for each project.

Each project builds on experience from the previous project. It is reasonable to
simply start each project in the same git repository where you left off on the
previous (though you may want to add a [git tag] indicating where the previous
project ended).

[git tag]: https://git-scm.com/book/en/v2/Git-Basics-Tagging


## Course structure

The overall arc of the course is defined by a series of projects that
incrementally introduce new subjects important to systems programming in Rust.

Because building an entire database from scratch while also learning all the
concepts involved is a daunting task, each project is proceeded by a "building
blocks" section, where individual concepts are explored individually. These
building blocks will consist of external readings, external programming
problems, and other single-subject content.

The building blocks sections are an opportunity to clear your mind, step away
from the larger project, and focus your attention on learning and practicing a
single subject in isolation. **Do not pass them up**.

Each project builds on the last &mdash; the APIs, command-line interfaces, and
parts of the implementation will often remain the same between projects, while
you only integrate new features and improvements related to a project's subject
matter. As such, it is ok to begin each project by copying your source code from
the last.

The projects live in the [`projects` subdirectory][psd], each in their own
directories, and consist of a project description in `project.md`, and a Cargo
project containing the project framework and test suite. Each
project comes with a test suite, and the project should not be considered
finished until it passes with `cargo test`.

You will recieve further instruction about setting up the source code and test
suite, as well as project specifications, as you progress through the individual
projects.

The expected time to complete each section is not currently well-estimated, but
the building blocks might take up to 8 hours, and the projects up to 40 hours.
If you are spending much less time than that, or are spending more time, don't
worry: these are just bad estimates, and everybody's experience is different.


## Getting help

You will run into problems that you don't know how to solve. Don't suffer in
silence. Everybody needs help sometimes.

And fortunately the Rust community is amazing and welcoming, and wants to help
you. As you are progressing through this course, you are _strongly_ encouraged
to join the Rust community experience.

Here are the resources you should consider turning to first:

- The #rust-training channel on the [TiKV Slack]. This channel exists just for
  this course. Please consider joining to support your fellow students and other
  course-takers. There will always be someone there to answer questions, but
  there may be a delay due to time zones and other factors. Both English and
  Chinese languages are welcome here.

- The #beginners channel on the official [Rust Discord]. You are almost
  guaranteed to get some answer here, and if not, don't hesitate to ask again.
  The people who hang out here are there specifically to help. Because of time
  zone differences it may take time for somebody to respond. Only English will
  be consistently understood here.

- The QQ Rust group #1 ([QR code][qq]). For Chinese students, this is one of the
  major Rust communities in China. This is a great place to hang out generally,
  and for those unconfident in their English skills, a great place to ask
  for help. There are also WeChat groups, bit with their low population caps
  and invite requirements are more difficult to deal with here.

- The QQ Rust group #2 ([QR code][qq2]). Like the above. If group 1 is
  at capacity you can get into this one.

These resources may also be helpful:

- The official [users forum]. Apply the "help" tag to your post. Questions
  usually recieve an answer here, but the responses can be limited.

- [StackOverflow]. Apply the "rust" tag. You may or may not recieve a satisfying
  answer.

You are also welcome to email the primary author of this course, [Brian
Anderson][brson], at brian@pingcap.com. I will happily answer questions, and am
eager to hear your experience with the course.

Finally, if there is a [Rust user group] near you, go check it out. As a Rust programmer
these groups are where you will build some of your strongest connections. (Note
that that link goes to the old Rust website and may not be up to date).


## Making PNA Rust better

<!-- TODO At the end of each project is a link to a survey about your experience. Please
take a few minutes to complete it, and be comfortable expressing the challenges
you faced, and your criticisms of the course. The survey results are not seen by
anybody but the [primary author of the course][author], though aggregate
statistics may be reported publicly. -->

As you work through the course content, please be on the lookout for things you
would improve about the course, and either [submit issues][si] explaining, or
[submit pull requests][spr] with improvements. (If you are being graded,
accepted pull requests to this or any other repo used in the course _may_ count
toward extra credit during final evaluation &mdash; let your instructor or
evaluator know!). This is an opportunity to gain experience contributing to an
open-source Rust project. Make this a better course for the next person to take
it than it was for you.

See [CONTRIBUTING.md] for more.


## Practical Networked Applications in Rust

This is an outline of the course. Clicking each header will take you to the
instructions for that section.

### [Building Blocks 1][b1]

**Topics**: CLI programming, the cargo manifest and environment variables,
documenting Rust projects, rustup.


### [Project 1: The Rust toolbox][p1]

**Task**: Create an in-memory key/value store that passes simple tests and responds
to command-line arguments.

**Goals**:

- Install the Rust compiler and tools
- Learn the project structure used throughout this course
- Use `cargo init` / `run` / `test` / `clippy` / `fmt`
- Learn how to find and import crates from crates.io
- Define an appropriate data type for a key-value store

**Topics**: testing, the `clap` crate, `CARGO_VERSION` etc., the `clippy` and
  `rustfmt` tools.

**Extensions**: the `structopt` crate.


### [Building Blocks 2][b2]

**Topics**: Log-structured file I/O, the bitcask algorithm, Rust error handling,
comparing collection types.


### [Project 2: Log-structured file I/O][p2]

**Task**: Create a persistent key/value store that can be accessed from the
command line.

**Goals**:

- Handle and report errors robustly
- Use serde for serialization
- Write data to disk as a log using standard file APIs
- Read the state of the key/value store from disk
- Map in-memory key-indexes to on-disk values
- Periodically compact the log to remove stale data

**Topics**: log-structured file I/O, bitcask, the `failure` crate, `Read` /
`Write` traits, the `serde` crate.


### [Building Blocks 3][b3]

**Topics**: TCP, logging, traits, benchmarking.


### [Project 3: Synchronous client-server networking][p3]

**Task**: Create a single-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Create a client-server application
- Write a custom protocol with `std` networking APIs
- Introduce logging to the server
- Implement pluggable backends via traits
- Benchmark the hand-written backend against `sled`

**Topics**: `std::net`, logging, traits, benchmarking.

**Extensions**: shutdown on signal.


### [Building Blocks 4][b4]

**Topics**: Message passing, lock-free data structures.


### [Project 4: Parallelism][p4]

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread-pool
- Use channels for cross-thread communication
- Share data structures with locks
- Perform compaction in a background thread
- Share data structures without locks
- Benchmark single-threaded vs multi-threaded

**Topics**: threads, thread-pools, channels, locks.


### [Building Blocks 5][b5]

**Topics**: Sync vs. async, Rust futures, tokio, `impl Trait`, existential types.


### [Project 5: Asynchronous programming in Rust][p5]

**Task**: Create a multi-threaded, persistent key/value store server and client
with asynchronous networking over a custom protocol.

**Goals**:

- Understand the patterns used when writing Rust futures
- Understand error handling with futures
- Learn to debug the type system
- Perform asynchronous networking with the tokio runtime
- Use boxed futures to handle difficult type-system problems
- Use `impl Trait` to create anonymous `Future` types

**Topics**: asynchrony, futures, tokio, `impl Trait`.

**Extensions**: tokio-fs.


<!-- building block links -->

[b1]: building-blocks/bb-1.md
[b2]: building-blocks/bb-2.md
[b3]: building-blocks/bb-3.md
[b4]: building-blocks/bb-4.md
[b5]: building-blocks/bb-5.md


<!-- project links -->

[p1]: projects/project-1/project.md
[p2]: projects/project-2/project.md
[p3]: projects/project-3/project.md
[p4]: projects/project-4/project.md
[p5]: projects/project-5/project.md


<!-- other links -->

[CONTRIBUTING.md]: ./CONTRIBUTING.md
[README.md]: ./README.md
[Rust Discord]: https://discord.gg/rust-lang
[Rust user group]: https://prev.rust-lang.org/en-US/user-groups.html
[StackOverflow]: https://stackoverflow.com/questions/tagged/rust
[TiKV Slack]: https://join.slack.com/t/tikv-wg/shared_invite/enQtNTUyODE4ODU2MzI0LTgzZDQ3NzZlNDkzMGIyYjU1MTA0NzIwMjFjODFiZjA0YjFmYmQyOTZiNzNkNzg1N2U1MDdlZTIxNTU5NWNhNjk
[author]: https://github.com/brson/
[brson]: https://github.com/brson/
[kv]: https://en.wikipedia.org/wiki/Key-value_database
[pre]: ./README.md#user-content-prerequisites
[psd]: https://github.com/pingcap/talent-plan/tree/master/rust/projects
[qq]: ./qq-qr.jpg
[qq2]: ./wechat-qr.jpg
[rs]: https://github.com/pingcap/talent-training/rust
[si]: https://github.com/pingcap/talent-plan/issues
[spr]: https://github.com/pingcap/talent-plan/pulls
[users forum]: https://users.rust-lang.org/
