# Answers for project "tools and good bones"

## Question A

> What does this directory layout suggest about how `lib.rs`, `main.rs`, and
`tests.rs` are compiled, and how they are linked to each other?

Answer: The compilation of our library file `lib.rs` is independent from
`main.rs` and `tests.rs`. The library becomes an implicit dependency of
`main.rs` and `tests.rs`, and is linked to the binaries `main.rs` and
`tests.rs` compile into.

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

Answer: `cargo test` run unit tests, integration tests and documentation
tests. In our case, the four test suites include two unit test suites from
the binary and the library respectively, one integration test suite from
`tests/tests.rs` and one documentation test from the library only.

## Question D
> Even if a given test suite doesn't contain any tests, why
might we not want them to run? Besides issuing the above command, how could
we permanently disable the three test suites we don't care about by
editing the project manifest (`Cargo.toml`)?

Answer: An empty test suite also needs the library to be built and linked
to it. So we can use `cargo test` to make sure our library can be
successfully built and linked.

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

Answer: A library is usually used by various users and applications.
If the user interface is placed into the library, we tend to hide
everything else in our library because our binary need not know them.
Not exposing public APIs prevents us from distributing the library to other
users who need the functions inside and do not need the user interface.
Even though there are necessary public APIs, the user interface is definitely
a waste of space and confusing to other users.