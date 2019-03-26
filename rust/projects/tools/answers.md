# Answers for project "tools and good bones"

## Question A

> What does this directory layout suggest about how `lib.rs`,
  `main.rs`, and `tests.rs` are compiled, and how they are linked to each other?
  How many libraries and executables will this project build? Which source files
  are compiled into each library and executable? Which executables link to which
  libraries?

Answer: The default library file is `src/lib.rs`. Executable source code
can be placed in `src/bin/*.rs`. Integration tests go in the `tests` directory.
If you organize your code this way, cargo will know which type each file is.

Cargo compiles `src/lib.rs` into a library. `src/bin/main.rs` can be built
into an executable with the library linked to it. `tests/tests.rs` is compiled
into a test binary via `cargo test`. Of course, the library is linked to the
test binary, too.

## Question B

> In practice we might not name `src/bin/main.rs` as `main.rs`. Why not? What's
the name of our binary? What are two ways we could change the name of that
binary?

Answer: If you have tried `cargo run`, you should have seen this line:

```
     Running `target/debug/main`
```

It says the executable name we are running is `main`, which is identical to
the name of the source file. In general, we expect our binary name to be
meaningful instead of `main`.

The simplest way to change the name is to just rename the source file.
For example, rename `main.rs` to `kvs.rs` and recompile, you will find the
binary name changes to `kvs` accordingly.

You can also configure it in `Cargo.toml` by adding a `[[bin]]` section:

```toml
[[bin]]
name = "kvs"
path = "src/bin/main.rs"
```

In fact, if you move `src/bin/main.rs` to `src/main.rs`, the binary name
will be the same as the package name (`kvs` in this case).

## Question C

> Notice that there are _four_ different set of tests running (each could be
called a "test suite"). Where do each of those test suites come from?

Answer: `cargo test` runs unit tests, integration tests and documentation
tests. In our case, the four test suites include two unit test suites from
the binary and the library respectively, one integration test suite from
`tests/tests.rs` and one documentation test from the library only.

## Question D
> Why might we not want to run empty test suites? Besides issuing
  the above command, how could we permanently disable the three test suites we
  don't care about by editing the project manifest (`Cargo.toml`)?

Answer: An empty test suite also needs the library to be built and linked
to it. It can be more and more time consuming as the project grows.

Unit tests and doc tests can be disabled in `Cargo.toml`.
Here is a possible answer:

```toml
[lib]
test = false
doctest = false

[[bin]]
name = "main"
test = false
```


## Question E
> This pattern of having the executable do nothing but call into
the library's `main` function is often a reasonable thing to do, though it is
not often used for production libraries. What are some downsides of placing a
program's user interface into a library instead of directly into the executable?

Answer: If you put the UI in the library, the library consumers are going to be
forced to link to the UI code along with the dependencies that are only needed
by the UI. This will increase their compilation time and binary size.