# PNA Rust Lesson plan

A training course about practical systems software construction in Rust.

Over a series of projects, you will build a networked [key-value database][kv],
with multithreading and asynchronous I/O. In between projects you will study and
practice individual subjects necessary to complete the next project. Along the
way you will explore multiple designs and their tradeoffs.

<!-- NOTE: keep the above in sync with README.md -->

See [README.md] for the overview, goals, audience, and prerequisites.

- [Prerequisites](#user-content-prerequisites)
- [Getting the materials](#user-content-getting-the-materials)
- [Course structure](#user-content-course-structure)
- [Getting help](#user-content-getting-help)
- [Making PNA Rust Better](#user-content-making-pna-rust-better)
- [Practical Networked Applications in Rust](#user-content-practical-networked-applications-in-rust)
  - [Building Blocks 1](#user-content-building-blocks-1)
  - [Project 1: The Rust toolbox](#user-content-project-1-the-rust-toolbox)
  - [Building Blocks 2](#user-content-building-blocks-2)
  - [Project 2: Log-structured file I/O](#user-content-project-2-log-structured-file-io)
  - [Building Blocks 3](#user-content-building-blocks-3)
  - [Project 3: Synchronous client-server networking](#user-content-project-3-synchronous-client-server-networking)
  - [Building Blocks 4](#user-content-building-blocks-4)
  - [Project 4: Concurrency and Parallelism](#user-content-project-4-concurrency-and-parallelism)
  - [Building Blocks 5](#user-content-building-blocks-5)
  - [Project 5: Asynchronous programming in Rust](#user-content-project-5-asynchronous-programming-in-rust)
- [What next?](#user-content-what-next)


## Prerequisites

As [described in the README.md][pre], this is not a course for novice
programmers, and there are significant prerequisites. Ensure that you meet them
all before proceeding.


## Getting the materials

All material for this course is in the

> https://github.com/pingcap/talent-plan

git repository on GitHub, in the [`rust` subdirectory][rs]. You will want a copy
of it on your local computer, particularly for easy access to the conformance
tests for each project.


## Course structure

The overall arc of the course is defined by a series of coding projects that
incrementally introduce new subjects important to systems programming in Rust.
Each project extends the previous project, so it is reasonable to simply start
your work on each project in the same git repository where you left off on the
previous (though you may want to add a [git tag] indicating where the previous
project ended).

[git tag]: https://git-scm.com/book/en/v2/Git-Basics-Tagging

Because building an entire database from scratch while also learning all the
concepts involved is a daunting task, each project is preceded by a "building
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
project containing a complete example solution, and the project test suite. Each
project comes with a test suite, and the project should not be considered
finished until it passes with `cargo test`.

**You should not read the example code for any project until completing the
project yourself** &mdash; good learning requires trying on your own and failing, over
and over until you succeed. You are encouraged though to learn from and apply
techniques they contain to your own projects retroactively. Keep in mind though
that the example projects are not the only way &mdash; or even the best way
&mdash; to solve the problems. Trust yourself and your own creativity.

<!-- TODO this is pretty harsh

> RE plagiarism (this is mostly relevant to students being evaluated on their
coursework): The line between applying techniques learned by code-reading and
copying code outright can be hard to identify. But as a professional you have
ethical responsibilites, and only you can know if you are upholding them. For
those not being evaluated for their coursework, simply copying from the example
isn't ; for those who are being evaluated, your instructors and evaluators are
expecting you to use your own skills.

-->

You will receive further instruction about setting up the source code and test
suite, as well as project specifications, as you progress through the individual
projects.

The expected time to complete each section is not currently well-estimated, but
both "building blocks" and "project" will probably take hours, not days, with
the projects taking more time. If you are spending much less time than that, or
are spending more time, don't worry: these are just bad estimates, and
everybody's experience is different.


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

- The `#beginners` channel on the official [Rust Discord]. You are almost
  guaranteed to get some answer here, and if not, don't hesitate to ask again.
  The people who hang out here are there specifically to help. Because of time
  zone differences it may take time for somebody to respond. Only English will
  be consistently understood here.

- The QQ Rust group #1 ([QR code][qq]). For Chinese students, this is one of the
  major Rust communities in China. This is a great place to hang out generally,
  and for those unconfident in their English skills, a great place to ask
  for help. There are also WeChat groups, but with their low population caps
  and invite requirements, they are more difficult to deal with here.

- The QQ Rust group #2 ([QR code][qq2]). Like the above. If group 1 is
  at capacity you can get into this one.

These resources may also be helpful:

- The official [users forum]. Apply the "help" tag to your post. Questions
  usually receive an answer here, but the responses can be limited.

- [StackOverflow]. Apply the "rust" tag. You may or may not receive a satisfying
  answer.

You are also welcome to email the primary author of this course, [Brian
Anderson][brson], at brian@pingcap.com. I will happily answer questions, and am
eager to hear your experience with the course.

Finally, if there is a [Rust meetup] near you, go check it out. As a Rust programmer
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
documenting Rust projects.


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

**Topics**: log-structured file I/O, the bitcask algorithm, Rust error handling,
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

**Topics**: unstructured vs. structured logging, the Redis protocol,
  benchmarking.


### [Project 3: Synchronous client-server networking][p3]

**Task**: Create a single-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Create a client-server application
- Write a custom protocol with `std` networking APIs
- Introduce logging to the server
- Implement pluggable backends with traits
- Benchmark the hand-written backend against `sled`

**Topics**: `std::net`, logging, traits, benchmarking.


### [Building Blocks 4][b4]

**Topics**: multithreading, thread pools, aliasing and mutability, concurrent
data types.


### [Project 4: Concurrency and parallelism][p4]

**Task**: Create a multithreaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread pool
- Use channels for cross-thread communication
- Share data structures with locks
- Perform read operations without locks
- Benchmark single-threaded vs multithreaded

**Topics**: thread pools, channels, locks, lock-free data structures,
  atomics, parameterized benchmarking.


### Building Blocks 5

Coming soon! ([preview][b5])


### Project 5: Asynchronous programming in Rust

Coming soon! ([preview][p5])


## What next?

So you have completed Practical Networked Applications in Rust. That's a Rusty
accomplishment! Now you are on the path to being a great Rust programmer. Want
to know where to go next on that path? We've got [some ideas][n].


<!-- building block links -->

[b1]: ../building-blocks/bb-1.md
[b2]: ../building-blocks/bb-2.md
[b3]: ../building-blocks/bb-3.md
[b4]: ../building-blocks/bb-4.md
[b5]: ../building-blocks/bb-5.md


<!-- project links -->

[p1]: ../projects/project-1/project.md
[p2]: ../projects/project-2/project.md
[p3]: ../projects/project-3/project.md
[p4]: ../projects/project-4/project.md
[p5]: ../projects/project-5/project.md


<!-- other links -->

[CONTRIBUTING.md]: ../CONTRIBUTING.md
[README.md]: ../README.md
[Rust Discord]: https://discord.gg/rust-lang
[Rust meetup]: https://www.meetup.com/topics/rust
[StackOverflow]: https://stackoverflow.com/questions/tagged/rust
[TiKV Slack]: https://join.slack.com/t/tikv-wg/shared_invite/enQtNTUyODE4ODU2MzI0LWVlMWMzMDkyNWE5ZjY1ODAzMWUwZGVhNGNhYTc3MzJhYWE0Y2FjYjliYzY1OWJlYTc4OWVjZWM1NDkwN2QxNDE
[author]: https://github.com/brson/
[brson]: https://github.com/brson/
[kv]: https://en.wikipedia.org/wiki/Key-value_database
[pre]: ../README.md#user-content-prerequisites
[psd]: https://github.com/pingcap/talent-plan/tree/master/rust/projects
[qq]: ./qq-qr.jpg
[qq2]: ./qq2-qr.jpg
[rs]: https://github.com/pingcap/talent-plan/tree/master/rust
[si]: https://github.com/pingcap/talent-plan/issues
[spr]: https://github.com/pingcap/talent-plan/pulls
[users forum]: https://users.rust-lang.org/
[n]: ./what-next.md
