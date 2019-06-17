# PNA Rust &mdash; Building Blocks 2

Let's learn some building blocks!

Put your other projects and concerns aside. Take a breath and relax. Here
are some fun resources for you to explore.

Read all the readings and perform all the exercises.

- **[Reading: Damn Cool Algorithms: Log structured storage][lss]**. A simple
  overview of the basic concept of log-structured storage. There are many
  log-structured storage algorithms, and the particular one described here is
  not the one you will be using.

- **[Reading: The Design and Implementation of a Log-Structured File
  System][lsfs]**. The influential paper.

- **[Reading: Bitcask: A Log-Structured Hash Table for Fast Key/Value Data][bc]**.
  A simple but effective design for a key-value database, and one that uses
  log-structured storage.

- **[Reading: Error Handling in Rust][e]**. Rust error handling is powerful, and
  many Rust programmers adore it once they have gotten the hang of it. But it is
  complex, and has a complex history of trial and error. This is a classic
  in-depth article on best-practices for error handling in Rust. It is from
  2015, and there have been some minor changes to error handling since then, but
  there is a lot of wisdom in here. The author, [BurntSushi], has done much
  experimenting with Rust error handling, and is considered an authority on that
  and [other things].

- **[Reading: `std::collections`][c]**. As a systems programmer it is crucial to know
  the behavior of a variety of data structures well, if not their
  implementations. The standard library's `collections` module has a pretty
  amazing overview of the tradeoffs a number of the most common collection types
  in software engineering. For this, you only need to read the module docs.

- **[Reading: `std::io`][io]**. Again, you need to know your I/O tools, and
  again the Rust `io` module docs, while not as great reading as the
  `collections` docs, provide a good overview of your toolset. You only need to
  read the module docs.

- **Exercise: Serialize and deserialize a data structure with `serde` (JSON)**.
  Imagine a flat game-playing surface covered in a grid of squares, like a chess
  board. Imagine you have a game character that every turn may move any number
  of squares. Define a type, `Move` that represents a single move of that
  charecter.

  Derive the [`Debug`] trait so `Move` is easily printable with the `{:?}`
  format specifier.

  Write a `main` function that defines a variable, `a`, of type `Move`,
  serializes it with [`serde`] to a variable, `v`, of type `Vec<u8>`, then
  deserializes it back again to a variable, `b`, also of type `Move`.

  Use [JSON] as the serialization format.

  Then, to see the representation at each step, use `println!` with `{:?}` to
  print `a`, `v` and `b`.

  Finally, convert the `Vec<u8>` to `String` with [`str::from_utf8`], unwrapping
  the result, then print that serialized string representation to see what
  `Move` looks like serialized to `JSON`.

  Note that the `serde` book has many [examples] to work off of.

- **Exercise: Serialize and deserialize a data structure to a file with `serde` (RON)**.

  Do the same as above, except this time, instead of serializing to a `Vec`,
  serialize to a file, and instead of JSON as the format, use [RON].

- **Exercise: Serialize and deserialize 1000 data structures with `serde` (BSON)**.

  This one is slightly different. Just serialize and deserialize 1000 `Move`
  values sequentially to and from a single `Vec<u8>`. This time use the [BSON]
  format.

[BSON]: https://github.com/zonyitoo/bson-rs
[RON]: https://github.com/ron-rs/ron
[`str::from_utf8`]: https://doc.rust-lang.org/std/str/fn.from_utf8.html
[JSON]: https://github.com/serde-rs/json
[`Debug`]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
[examples]: https://serde.rs/examples.html
[`serde`]: https://serde.rs/
[lss]: http://blog.notdot.net/2009/12/Damn-Cool-Algorithms-Log-structured-storage
[lsfs]: https://people.eecs.berkeley.edu/~brewer/cs262/LFS.pdf
[io]: https://doc.rust-lang.org/std/io/
[c]: https://doc.rust-lang.org/std/collections/
[e]: https://blog.burntsushi.net/rust-error-handling/
[bc]: https://github.com/basho/bitcask/blob/develop/doc/bitcask-intro.pdf
[BurntSushi]: https://github.com/BurntSushi
[other things]: https://github.com/BurntSushi/ripgrep

<!-- TODO: better LSS paper -->
<!-- TODO: want a general non-wikipedia survey of how databases and/or key/value dbs work -->
