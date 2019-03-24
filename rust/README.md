# Practical Networked Applications in Rust

A training course on practical software construction in Rust.

The final project in this course is a networked, persistent [key-value store]
with multithreading, asynchronous I/O (via [hyper]).

Subjects covered include:

- Structuring and maintaining Rust programs
- Fun and foolproof parallel programming
- Asyncronous programming with [futures]
- Networking with [hyper]
- Benchmarking with [criterion] and [critcmp]
- Robust and reliable error handling with [failure]
- Serialization with [serde]
- Build scripts and the interaction between build-time and run-time
- Insights into inner workings of the language and libraries
- How to learn what you don't know about Rust and find what you need to find

Goals of this course are to convey knowledge relevant to real-world Rust
programming, including inherited wisdom and best practices; to look "under the
hood" and understand how and why Rust works like it does; to tour the world of
Rust documentation and code such that the student can learn how to find answers
to their questions on their own.

Non-goals of this course include teaching installation, syntax and other Rust
basics; teaching basic data structures and algorithms; being a comprehensive
resource on the Rust language. That information is easily found in [The Rust
Book].

The course can be followed and completed online or delivered offline as a series
of training sessions, and consists of

- _readings_ - background material that is expected to be read before each
  lesson
- _lessons_ - focused coverage of subjects required for writing practical
  software of the type developed in the corresponding projects, including
  advanced tips, best practices, and deep dives. This are in the form of slides,
  speaker notes, and short _writeups_.
- _projects_ - practical exercises that build upon each other to construct a
  final, testable and gradable project. Includes extra credit problems and
  anwsers.

[View the lesson plan][plan].

## Audience - who is this course for?

_This book is not for novice programmers, but it is for novice Rust programmers._

This course is for programmers with some knowledge of data structures,
algorithms and I/O.

This course is for programmers who have begun learning Rust, but still need
guidance in production Rust programming and best practices.

This is a course for active learners &mdash; it will lead you in the right
direction, but the journey is yours.

Those who complete the course will have the knowledge and experience to begin to
write high performance, reliable, networked applications in Rust.

This course, combined with [Deep Dive TiKV] is intended to be enough to enable
programmers to meaningfully contribute to [TiKV] or any other Rust project. It
is most specifically designed to teach those in the Chinese Rust community
enough Rust to work on TiKV. The language used is intended to be simple so that
those who read only a little English can follow.

### Prerequisite knowledge

Students should:

- have intermediate-level experience in some programming language,
- be familiar with the terminal and command line,
- know how to use [git],
- have read [The Rust Book],
- have Rust installed and know how to compile and run Rust programs.

[View the lesson plan][plan].

## Other courses in this series

This course is part of a [series of courses] initiated by [PingCAP] to train
students, contributors, new hires, and existing employees in Rust for
distributed systems. Those who complete this one may wish to continue
to [Distributed Systems in Rust].

## How to use this course

Links to the _readings_, _lessons_, and _slides_ are in the [lesson plan][plan],
presented in the order they should be followed. See the lesson plan for
further suggestions on how to get the most out of this course.

All material for this course is in the

> https://github.com/pingcap/talent-training

git repository on GitHub, in the `rust` subdirectory. You may want a copy of it
on your local computer, particularly for easy access to the conformance tests
for each project.

All material for this course can be viewed as a static website, online at

> https://todo.later

or on your own local computer. To browse locally just open index.html in a web
browser.

Get started now - [view the lesson plan][plan].

## For instructors

This _might_ be (or become) a useful teaching resource. As of yet it has
not been tested at all in a live setting. There is no guidance within
as to how to teach this course. And for now it is definitely not
recommended to teach this course.

## Contributing

See [CONTRIBUTING.md].

## License

[CC-BY 4.0](https://opendefinition.org/licenses/cc-by/)


<!-- links -->

[key-value store]: todo
[tokio]: todo
[tower]: todo
[GRPC]: todo
[prost]: todo
[tower-grpc]: todo
[LSM-tree]: todo
[futures]: todo
[criterion]: todo
[critcmp]: todo
[failure]: todo
[Deep Dive TiKV]: todo
[TiKV]: todo
[git]: todo
[The Rust Book]: todo
[series of courses]: todo
[PingCAP]: todo
[Distributed Systems in Rust]: todo
[The Rust Book]: https://doc.rust-lang.org/book/
[plan]: plan.md
[serde]: todo
[CONTRIBUTING.md]: CONTRIBUTING.md
