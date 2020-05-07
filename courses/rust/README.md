# Practical Networked Applications in Rust

A training course about practical systems software construction in [Rust].

Over a series of projects, you will build a single networked, multithreaded, and
asynchronous Rust application. Creating this application, a [key-value
database][kv], will provide opportunities to exercise the best of the crate
ecosystem, a variety of concurrent data types, the world of async Rust,
interesting language features, and important Rust tools. In between projects are
small lessons and exercises on the subjects necessary to complete the next
project.

<!-- TODO make the above sparkle -->
<!-- NOTE: keep the above in sync with lesson-plan.md -->

Subjects covered include:

- Structuring and maintaining Rust programs
- Applying common tools like [clippy] and [rustfmt]
- Best practices in Rust error handling
- Serialization with [serde]
- Simple log-structured storage, inspired by [bitcask]
- Network programming with std and [tokio]
- Benchmarking with [criterion]
- Fun and foolproof parallel programming with [crossbeam] and more
- Asyncronous programming with Rust [futures]
- How to learn what you don't know about Rust and find the documentation and
  crates you need to succeed

After completing this course you will have the knowledge and experience to begin
writing high performance, reliable, systems software in Rust. And you might
discover that doing so is simpler than you expected.

_**Important note: Practical Networked Applications in Rust is in an alpha
state**. It contains bugs and its scope is limited. If you are taking it now you
are brave, but you are also an early tester and your feedback is greatly
appreciated. As you are following along please [file issues]<!-- TODO and
complete the [post-project surveys] -->. You are also encouraged to fix problems
on your own and submit pull requests. See [CONTRIBUTING.md] for details.<!-- See
[the roadmap] for details about future course subject matter.-->_

**[View the lesson plan][plan]**.


## The goal of this course

The goal of this course is to teach new Rust programmers how to build real-world
[systems programs][sp], with all the desirable Rust characteristics, including
high-performance, reliability, and easy concurrency; and to do so using the best
practices that might not be evident to newcomers.

Non-goals of this course include teaching installation, syntax, and other Rust
basics; teaching basic data structures and algorithms; teaching basic parallel
and asynchronous programming concepts; and being a comprehensive resource on the
Rust language. That information is easily found in [The Rust Book] and
elsewhere.

**[View the lesson plan][plan]**.


## Who is this for?

Practical Networked Applications in Rust is for novice _Rust_ programmers, but it
is not for novice programmers.

The primary audience of this course is recent graduates and near-graduates of
an undergraduate computer science program who are considering starting a career
as a Rust systems programmer. Others will also likely benefit, including
experienced developers without systems programming experience.


## Prerequisites

Those taking this course should:

- [ ] have the equivalent of an undergraduate computer science education,
- [ ] have intermediate-level experience in some programming language,
- [ ] be comfortable working in the terminal and command line,
- [ ] know how to use [git],
- [ ] have novice-level experience with [parallel programming] in some language,
- [ ] have novice-level experience with [asynchronous programming] in some language,
- [ ] have novice-level experience writing code to query a database, [SQL],
  [NoSQL], [NewSQL], [key-value][kv], or otherwise.
- [ ] **have read [The Rust Book] in its entirety**,
- [ ] have written _some_ Rust code, including the projects from the book:
  - [programming a guessing game],
  - [building a command-line program] and
  - [building a multithreaded web server].

To reiterate &mdash; read [The Rust Book] _before_ taking this course. It is not
necessary to have more than novice-level knowledge or experience with Rust, but
this course does not teach Rust basics.

If you can check all the above boxes then you are ready for this course. If not,
we have some [suggestions][pre] for how to learn the prerequisites.

Get started now - **[view the lesson plan][plan]**.


## Other courses in this series

This course is part of a [series of courses] initiated by [PingCAP] to train
students, contributors, new hires, and existing employees in Rust for
distributed systems. Those who complete this one may wish to continue
to [Distributed Systems in Rust].


## A PingCAP-specific note

This course, combined with [Deep Dive TiKV], and the [Distributed Systems in
Rust] course is intended to be enough to enable programmers to meaningfully
contribute to [TiKV]. It is most specifically designed to teach those in the
Chinese Rust community enough Rust to work on TiKV. The language used is
intended to be simple so that those who read only a little English can follow.
If you find any of the language difficult to understand please [file issues].


## Contributing

See [CONTRIBUTING.md].


## License

All text and code for this course is dual licensed [CC-BY 4.0] and [MIT]. You
may freely reuse any material here under the terms of either or both, at your
discretion.


<!-- links -->

[CONTRIBUTING.md]: CONTRIBUTING.md
[CC-BY 4.0]: https://opendefinition.org/licenses/cc-by/
[MIT]: https://opensource.org/licenses/MIT
[Deep Dive TiKV]: https://tikv.org/deep-dive/introduction/
[Distributed Systems in Rust]: https://github.com/pingcap/talent-plan/tree/master/courses/dss
[NewSQL]: https://en.wikipedia.org/wiki/NewSQL
[NoSQL]: https://www.thoughtworks.com/insights/blog/nosql-databases-overview
[PingCAP]: https://pingcap.com/
[SQL]: https://en.wikipedia.org/wiki/SQL
[The Rust Book]: https://doc.rust-lang.org/book/
[The Rust Book]: https://doc.rust-lang.org/stable/book/
[TiKV]: https://github.com/tikv/tikv/
[asynchronous programming]: todo
[bitcask]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
[building a command-line program]: https://doc.rust-lang.org/stable/book/ch12-00-an-io-project.html
[building a multithreaded web server]: https://doc.rust-lang.org/stable/book/ch20-00-final-project-a-web-server.html
[clippy]: https://github.com/rust-lang/rust-clippy/
[criterion]: https://github.com/bheisler/criterion.rs
[crossbeam]: https://github.com/crossbeam-rs/crossbeam
[file issues]: https://github.com/pingcap/talent-plan/issues/
[futures]: https://docs.rs/futures/0.1.27/futures/
[git]: https://git-scm.com/
[kv]: https://en.wikipedia.org/wiki/Key-value_database
[parallel programming]: todo
[plan]: ./docs/lesson-plan.md
[post-project surveys]: ./docs/lesson-plan.md#user-content-making-pna-rust-better
[pre]: ./docs/prerequisites.md
[programming a guessing game]: https://doc.rust-lang.org/stable/book/ch02-00-guessing-game-tutorial.html
[rustfmt]: https://github.com/rust-lang/rustfmt/
[serde]: https://github.com/serde-rs/serde
[series of courses]: https://github.com/pingcap/talent-plan/
[sp]: https://en.wikipedia.org/wiki/System_programming
[the roadmap]: ./docs/roadmap.md
[tokio]: https://github.com/tokio-rs/tokio
[Rust]: https://www.rust-lang.org/
