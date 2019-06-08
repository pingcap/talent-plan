# Practical Networked Applications in Rust

A training course about practical software construction in Rust.

The final project in this course is a networked, persistent [key-value store]
with multithreading and asynchronous I/O.

Subjects covered include:

- Structuring and maintaining Rust programs
- Applying common tools like [clippy] and [rustfmt]
- Best practices in Rust error handling
- Serialization with [serde]
- Simple log-based data storage, inspired by [bitcask]
- Network programming with std and [tokio]
- Benchmarking with [criterion] and [critcmp]
- Fun and foolproof parallel programming
- Asyncronous programming with Rust [futures]
- Robust and reliable error handling with [failure]
- How to learn what you don't know about Rust and find the documentation and
  crates you need to succeed

Those who complete the course will have the knowledge and experience to begin
writing high performance, reliable, networked applications in Rust.

_**Important note: Practical Networked Applications in Rust is in an alpha
state**. It contains bugs and its scope is limited. If you are taking it now you
are brave, but you are also an early tester and your feedback is greatly
appreciated. As you are following along please [file issues] and complete the
[post-project surveys]. You are also encouraged to fix problems on your own and
submit pull requests. See [CONTRIBUTING.md] for more about contributing to
Practical Networked Applications in Rust. See [roadmap.md] for details about
future course subject matter._

**[View the lesson plan][plan]**.


## Goals of this course

The goal of this course is to teach new Rust programmers how build real-world
Rust programs, with all the desirable Rust characteristics, including
high-performance, reliability, and easy concurrency; and to do so using the best
practices that might not be evident to new Rust programmers.

Non-goals of this course include teaching installation, syntax and other Rust
basics; teaching basic data structures and algorithms; teaching basic parallel
and asynchronous programming concepts; and being a comprehensive resource on the
Rust language. That information is easily found in [The Rust Book] and
[elsewhere][pre].

**[View the lesson plan][plan]**.


## Audience - who is this for?

Practical Networked Applications in Rust is not for novice programmers, but it
is for novice Rust programmers.

The primary audience for this course is recent graduates and near-graduates of
an undergraduate computer science program who are considering starting a career
as a Rust systems programmer. Others will also likely benefit, including
experienced developers without systems-programming experience.

As prerequisites, students should:

- [ ] have the equivalent of an undergraduate computer science education
- [ ] have intermediate-level experience in some programming language,
- [ ] be comfortable working in the terminal and command line,
- [ ] know how to use [git],
- [ ] have novice-level experience with [parallel programming] in some language,
- [ ] have novice-level experience with [asynchronous programming] in some language,
- [ ] **have read [The Rust Book] in its entirety**, and written _some_ Rust
  code, including the projects from the book:
  - [programming a guessing game],
  - [building a command-line program] and
  - [building a multithreaded web server].
- [ ] have Rust installed and know how to compile and run Rust programs.

To reiterate &mdash; read [The Rust Book] _before_ taking this course. It is not
necessary to have more than novice knowledge of Rust experience writing Rust,
but this course does not teach Rust basics.

If you can check all the above boxes then you are ready for this course. If not,
we have some [suggestions][pre] of where to look to learn the prerequisites.

Get started now - **[view the lesson plan][plan]**.


## Other courses in this series

This course is part of a [series of courses] initiated by [PingCAP] to train
students, contributors, new hires, and existing employees in Rust for
distributed systems. Those who complete this one may wish to continue
to [Distributed Systems in Rust].


## A PingCAP-specific note

This course, combined with [Deep Dive TiKV], and the [Distributed Systems in
Rust] course is intended to be enough to enable programmers to meaningfully
contribute to [TiKV] or any other Rust project. It is most specifically designed
to teach those in the Chinese Rust community enough Rust to work on TiKV. The
language used is intended to be simple so that those who read only a little
English can follow. If you find any of the language difficult to understand
please [file issues].


## Contributing

See [CONTRIBUTING.md].


## License

[CC-BY 4.0](https://opendefinition.org/licenses/cc-by/)


<!-- links -->

[key-value store]: https://en.wikipedia.org/wiki/Key-value_database
[tokio]: https://github.com/tokio-rs/tokio
[futures]: https://docs.rs/futures/0.1.27/futures/
[criterion]: https://github.com/bheisler/criterion.rs
[critcmp]: https://github.com/BurntSushi/critcmp
[failure]: https://github.com/rust-lang-nursery/failure
[Deep Dive TiKV]: https://tikv.org/deep-dive/
[TiKV]: https://github.com/tikv/tikv/
[git]: https://git-scm.com/
[The Rust Book]: https://doc.rust-lang.org/stable/book/
[series of courses]: https://github.com/pingcap/talent-plan/
[PingCAP]: https://pingcap.com/
[Distributed Systems in Rust]: https://github.com/pingcap/talent-plan/tree/master/dss
[The Rust Book]: https://doc.rust-lang.org/book/
[plan]: lesson-plan.md
[serde]: https://github.com/serde-rs/serde
[CONTRIBUTING.md]: CONTRIBUTING.md
[roadmap.md]: roadmap.md
[file issues]: https://github.com/pingcap/talent-plan/issues/new
[post-project surveys]: plan.md#surveys
[clippy]: https://github.com/rust-lang/rust-clippy/
[rustfmt]: https://github.com/rust-lang/rustfmt/
[programming a guessing game]: https://doc.rust-lang.org/stable/book/ch02-00-guessing-game-tutorial.html
[building a command-line program]: https://doc.rust-lang.org/stable/book/ch12-00-an-io-project.html
[building a multithreaded web server]: https://doc.rust-lang.org/stable/book/ch20-00-final-project-a-web-server.html
[pre]: prerequisies.md
[asynchronous programming]: todo
[parallel programming]: todo
[bitcask]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
