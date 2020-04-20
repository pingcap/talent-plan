# PNA Rust &mdash; Building Blocks 4

Let's learn some building blocks!

Put your other projects and concerns aside. Take a breath and relax. Here
are some fun resources for you to explore.

Read all the readings and perform all the exercises. Also watch the video.

- **[Reading: Fearless Concurrency with Rust][f]**. This is a classic Rust blog
  post from [Aaron Turon][at] that clearly explains why concurrency is so easy
  in Rust. The title is also the origin of using the word "fearless" to describe
  various Rust properties.
  
- **[Reading: What is the difference between concurrency and parallelism?][d]**.
  This is a 10 second read, but something to keep in mind. The two words are
  often used interchangably. We'll mostly use the word "concurrent" since it is
  more general than "parallel". Sometimes we'll use the word "parallel" to be
  more specific, sometimes because it sounds better…

- **[Reading: Rust: A unique perspective][ru]**. An explanation of the dangers
  of mutable aliasing and how Rust solves the problem. This one is by [Matt
  Brubeck][mb], from the [Servo] team.

- **[Video: Rust Concurrency Explained][ex]**. A more in-depth talk by [Alex
  Crichton][ac]. Aaron and Alex wrote many of the concurrent data structures in
  the standard library. Alex has given versions of this talk for years, and it
  is a pretty great overview of what Rust can do.

- **[Reading: `std::sync`][ss]**. Once again, the standard library documentation
  provides not only good documentation about the library, but about the subject
  in general. This provides an overview of most of the concurrent types provided
  by the standard library.

- **[Exercise: Basic multithreading][bmt]**. This is a simple multithreading
  exercise from the [rustlings] project. It is small enough that it can be
  completed on [play.rust-lang.org].

- **Exercise: Write a thread pool**.

  A [thread pool] runs jobs (functions) on a set of reusable threads, which can
  be more efficient than spawning a new thread for every job.

  Create a simple thread pool with the following type signature:

  ```rust
  impl ThreadPool {
    fn new(threads: u32) -> Result<Self>;

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
  }
  ```

  The `new` function should immediately spawn `threads` number of threads, and
  then those threads will wait for jobs to be spawned. When a thread recieves a
  job, it runs it to completion, then waits for the next job.

  The [`threadpool`][tp1] crate and Rayon's [`ThreadPool`][tp2] may provide inspiration.

- **[Reading: Lock-free vs wait-free concurrency][lf]**. It seems like everybody
  wants their code to be "lock-free". What's that mean?

<!--

TODO

overview of concurrent data structures
code reordering

https://en.wikipedia.org/wiki/Concurrent_data_structure
https://preshing.com/20120625/memory-ordering-at-compile-time/
https://www.cl.cam.ac.uk/~jp622/the_problem_of_programming_language_concurrency_semantics.pdf
concurrent maps https://gitlab.nebulanet.cc/xacrimon/rs-hm-bench

-->

[lf]: https://rethinkdb.com/blog/lock-free-vs-wait-free-concurrency/
[play.rust-lang.org]: https://play.rust-lang.org/
[tp1]: https://docs.rs/threadpool/1.7.1/threadpool/struct.ThreadPool.html
[tp2]: https://docs.rs/rayon/1.0.3/rayon/struct.ThreadPool.html
[thread pool]: https://softwareengineering.stackexchange.com/questions/173575/what-is-a-thread-pool#173581
[ss]: https://doc.rust-lang.org/std/sync/index.html
[Servo]: https://github.com/servo/servo
[mb]: https://github.com/mbrubeck/
[ru]: https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html
[ac]: https://github.com/alexcrichton/
[ex]: https://www.youtube.com/watch?v=Dbytx0ivH7Q
[f]: https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html
[d]: https://stackoverflow.com/questions/1050222/what-is-the-difference-between-concurrency-and-parallelism#1050257
[at]: https://github.com/aturon
[bmt]: https://github.com/rust-lang/rustlings/blob/master/exercises/threads/threads1.rs
[rustlings]: https://github.com/rust-lang/rustlings/
