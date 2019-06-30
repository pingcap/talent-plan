# Distributed Systems in Rust

A training course about the distributed systems in [Rust].

Subjects covered include:

- [Raft consensus algorithm] (including a fault-tolerant key-value storage service
using Raft)
- [Percolator transaction model]

After completing this course you will have the knowledge to implement a basic
key-value storage service with transaction and fault-tolerant in Rust.

**Important note: Distributed Systems in Rust is in an alpha state**
It might contain bugs. Any feedback is greatly appreciated. Please [file issues]
if you have any problem. And also You are encouraged to fix problems on your own
and submit pull requests.

## The goal of this course

The goal of this course is to teach the Rust programmers who are interested in
distributed systems to know about how to make the distributed systems reliable
and how to implement the distributed transaction.

## Who is this for?

Distributed Systems in Rust is for experienced _Rust_ programmers, who are
familiar with the Rust language. If you are not, you can first learn our [rust]
lessons.

## A PingCAP-specific note

This course, combined with [Deep Dive TiKV], is intended to be enough to enable
programmers to meaningfully contribute to [TiKV]. It is most specifically
designed to teach those in the Chinese Rust community enough Rust to work on
TiKV. The language used is intended to be simple so that those who read only a
little English can follow. If you find any of the language difficult to
understand please [file issues].

## License

[CC-BY 4.0](https://opendefinition.org/licenses/cc-by/)

<!-- links -->
[rust]: ../rust/README.md
[file issues]: https://github.com/pingcap/talent-plan/issues/
[Deep Dive TiKV]: https://tikv.org/deep-dive/
[TiKV]: https://github.com/tikv/tikv/
[Rust]: https://www.rust-lang.org/
[Raft consensus algorithm]: raft/README.md
[Percolator transaction model]: percolator/README.md
