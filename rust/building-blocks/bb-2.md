# PNA Rust &mdash; Building Blocks 2

Let's learn some building blocks!

Put your other projects and concerns aside. Take a breath and relax. Here
are some fun resources for you to explore.

- **[Reading: Damn Cool Algorithms: Log structured storage][lss]**. A simple
  overview of the basic concept of log-structured storage. Do note that the
  particular algorithm described here is not the one you will be using.

- **[Reading: Bitcask: A Log-Structured Hash Table for Fast Key/Value Data][bc]**.
  A simple but effective design for a key-value database, and one that uses
  log-structured storage. The one you are building will be inspired by it.

- **[Reading: Error Handling in Rust]**. Rust error handling is powerful, and
  many Rust programmers adore it once they have gotten the hang of it. But it is
  complex, and has a complex history of trial and error. This is a classic
  in-depth article on best-practices for error handling in Rust. It is from
  2015, and there have been some minor changes to error handling since then, but
  there is a lot of wisdom in here. The author, [BurntSushi], has done much
  experimenting with Rust error handling, and is considered an authority on that
  and [other things].

- **[Reading: std::collections]**. As a systems programmer it is crucial to know
  the behavior of a variety of data structures well, if not their
  implementations. The standard library's collections module has a pretty
  amazing overview of the tradeoffs a number of the most common collection types
  in software engineering.

[Reading: std::collections]: https://doc.rust-lang.org/std/collections/
[lss]: http://blog.notdot.net/2009/12/Damn-Cool-Algorithms-Log-structured-storage
[Reading: Error Handling in Rust]: https://blog.burntsushi.net/rust-error-handling/
[bc]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
[BurntSushi]: https://github.com/BurntSushi
[other things]: https://github.com/BurntSushi/ripgrep

<!-- TODO: better LSS paper -->
<!-- TODO: want a general non-wikipedia survey of how databases and/or key/value dbs work -->
