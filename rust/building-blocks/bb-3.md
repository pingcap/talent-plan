# PNA Rust &mdash; Building Blocks 3

Let's learn some building blocks!

Put your other projects and concerns aside. Take a breath and relax. Here
are some fun resources for you to explore.

Read all the readings and perform all the exercises.

- **[Reading: `log` crate API][l]**. The original Rust logging crate. Just read
  the crate-level documentation (the front page). You may need to click little `[+]`
  or `[-]` buttons to make the crate docs visible. This will give you an idea
  about how logging works in Rust.

- **[Reading: `slog` crate API][sl]**. Another popular logging crate, designed
  for "structured loging". Again, just read the crate-level docs, to compare to
  `log`. You might also want to look at ["Introduction to structured logging
  with slog"][sli].

- **[Reading: Benefits of Structured Logging vs basic logging][lvsl]**. A
  StackOverflow discussion about the differences between traditional,
  text-oriented, line logging and structured logging.

- **[Reading: Redis Protocol specification][rp]**. The protocol spec for
  [Redis], an in-memory key-value store. Think about what their design
  priorities were. When reading this it also helps to have the Redis [commands]
  on hand.

- **Exercise: Write a Redis ping-pong client and server using `std::io`**. Write
  a simple client and server that speaks the Redis protocol, with the client
  issuing [PING] commands and the server responding appropriately. Use the
  [`std::io`] APIs to read and write bytes directly. Does your client work with
  an actual Redis server?

- **Exercise: Write a Redis ping-pong client and server with serialized
  messages**. Same as above, but this time define the protocol with types and
  use [`serde` custom serialization][cs] to indirectly read and write messages
  through serialization.

- **[Reading: Statistically Rigorous Java Performance Evaluation][pe]**.
  Although it is specifically about Java, and discusses topics relevant to
  garbage-collected languages, it is a good example of the kind of thinking
  necessary to create effective benchmarks.

<!-- TODO: better benchmarking reading -->
<!-- TODO: something about traits? -->

[pe]: https://dri.es/files/oopsla07-georges.pdf
[cs]: https://serde.rs/custom-serialization.html
[`std::io`]: https://doc.rust-lang.org/std/io/
[PING]: https://redis.io/commands/ping
[commands]: https://redis.io/commands
[Redis]: https://redis.io/
[rp]: https://redis.io/topics/protocol
[l]: https://docs.rs/log/
[sl]: https://docs.rs/slog/
[sli]: https://github.com/slog-rs/slog/wiki/Introduction-to-structured-logging-with-slog
[lvsl]: https://softwareengineering.stackexchange.com/questions/312197/benefits-of-structured-logging-vs-basic-logging
