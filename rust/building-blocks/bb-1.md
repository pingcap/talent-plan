# PNA Rust &mdash; Building Blocks 1

Let's learn some building blocks!

Put your other projects and concerns aside. Take a breath and relax. Here
are some fun resources for you to explore.

Read all the readings and perform all the exercises.

- **[Exercise: Write a Good CLI Program]**. Writing a CLI program in Rust. This
  is a good warmup to the CLI program you'll be writing in this course, and the
  techniques used by this author may provide an interesting contrast to those we
  suggest. Follow along and write the same code. Can you reproduce their
  results?

- **[Reading: The Cargo manifest format]**. A single page in [The Cargo Book],
  this will give you an idea of how your project can be customized a bit if you
  so choose. This is a page you will come back to repeatedly as a Rust
  programmer.

- **[Reading: Cargo environment variables]**. Also from The Cargo Book, and also
  a page that you will see many times in the future. Environment variables are
  one way that it communicates with rustc, allowing it to set the various
  [`env!`] macros at build time, in both your program source code and build
  scripts. It is also a way for scripts and other systems to communicate to
  Cargo.

- **[Reading: Rust API Guidelines: Documentation]**. The Rust project is
  opinionated about how Rust source is written. This page is on how to document
  Rust projects, but the whole book is worth reading. These are written by
  experienced Rust developers, but are in an incomplete state. Note the GitHub
  organization it belongs to &mdash; [`rust-lang-nursery`]. It contains many
  interesting projects.


[Reading: Rust API Guidelines: Documentation]: https://rust-lang-nursery.github.io/api-guidelines/documentation.html
[Reading: The Cargo manifest format]: https://doc.rust-lang.org/cargo/reference/manifest.html
[Reading: Cargo environment variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html
[The Cargo Book]: https://doc.rust-lang.org/cargo/reference/manifest.html
[`env!`]: https://doc.rust-lang.org/std/macro.env.html
[`rust-lang-nursery`]: https://github.com/rust-lang-nursery
[Reading: The rustup documentation]: https://github.com/rust-lang/rustup.rs/blob/master/README.md
[Exercise: Write a Good CLI Program]: https://qiita.com/tigercosmos/items/678f39b1209e60843cc3

