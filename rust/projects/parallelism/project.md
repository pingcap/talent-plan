# Project: Parallelism

**Task**: Create a multi-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- Write a simple thread-pool
- Use crossbeam channels
- Share data structures with locks
- Perform compaction in a background thread
- Share data structures without locks
- Benchmark single-threaded vs multi-threaded

**Topics**: threads, thread-pools, channels, locks, basic tokio


## Introduction

In this project you will create a simple key/value server and client that
communicate over a custom protocol. The server will use synchronous networking,
and will respond to multiple requests using increasingly sophisticated
parallel implementations. The in-memory index will become a concurrent
data structure, shared by all threads, and compaction will be done on a
dedicated thread, to reduce latency of individual requests.

This is the final project before we get to the real heavy stuff &mdash; async.
Be afraid...


## Project spec

The cargo project, `kvs`, builds a command-line key-value store client called
`kvs-client`, and a key-value store server called `kvs-server`, both of which in
turn call into a library called `kvs`. The client speaks to the server over
a custom protocol.

The interface to the CLI is the same as in the [previous project]. The
difference this time is in the parallel implementation, which will be described
as we work through it..

The library interface is nearly the same except for two things. First this time
all the `KvsEngine`, `KvStore`, etc. methods take `&self` instead of `&mut
self`, and now it implements `Clone`. This is common with parallel
data structures. Why is that? It's not that we're not going to be writing
immutable code. It _is_ though going to be shared across threads. Why might that
preclude using `&mut self` in the method signatures? If you don't know now,
it'll become obvious by the end of this project.

The second is that the library in this project contains a new _trait_,
`ThreadPool`. It contains the following methods:

- `ThreadPool::new(threads: u32) -> Result<ThreadPool>`

  Creates a new thread pool, immediately spawning the specified number of
  threads.

  Returns an error if any thread fails to spawn. All previously-spawned threads
  are terminated.

- `ThreadPool::spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static`

  Spawn a function into the threadpool.

  Spawning always succeeds, but if the function panics the threadpool continues
  to operate with the same number of threads &mdash; the thread count is not
  reduced nor is the thread pool destroyed, corrupted or invalidated.

By the end of this project there will be several implementations of this
trait, and you will again perform benchmarking to compare them.

This project should not require any changes at all to the client code.


## Project setup

Create a new cargo project and copy the `tests` directory into it. This project
should contain a library named `kvs`, and two executables, `kvs-server` and
`kvs-client`. As with previous projects, add enough definitions that the
test suite builds.


## Part 1: Multithreading

Your first try at introducing parallelism is going to be the simplest: spawning
a new thread per incoming connection, and responding to the request on that
connection, then letting the thread exit. What performance benefits will
distributing work across threads provide? How do you expect latency will be
affected? What about throughput?

The first step is to write a `ThreadPool` implementation for this naive
approach. Call it `NaiveThreadPool` (it's not _really_ even a thread _pool_
since the threads are not reused, but it needs to conform to our trait for later
comparisons).

We aren't focusing on a more sophisticated implementation now because simply
integrating this solution into our existing design is going to take some effort.
So note that the `ThreadPool::new` constructor takes a `threads` argument
specifying the number of threads in the pool. In this implementation it will go
unused.

_Go ahead and implement this version of `ThreadPool` now_, then we'll
integrate it into the new `KvStore`.

**Test cases to complete**:

  - `naive_thread_pool_*`


## Part 2: Creating a shared `KvEngine`

Before we can integrate the `NaiveThreadPool` into `KvServer` we have to make
the `KvEngine` trait and the `KvStore` implementation (for now you can ignore
the `SledKvsEngine` from the previous project, but you can optionally
re-implement it as an extension to this project).

Recall from the project spec that, this time, our `KvsEngine` takes `self` as
`&self` instead of `&mut self` as previously, and it also implements `Clone`.
More concretely, it looks like

```rust
pub trait KvsEngine: Clone {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;
}
```

This gives us a lot of clues about the implementation strategy we're pursuing.
First, think about why the engine needs to implement `Clone` when we have a
multi-threaded implementation. This once I'll just tell you that each thread is
going to need access to the engine. Now think about why that makes us use
`&self` instead of `&mut self`. What do you know about shared mutable state? By
the end of this project be sure you understand the implications here &mdash;
_this is what Rust is all about_.

In this model, `KvsEngine` behaves like a _handle_ to another object, and
because that object is shared between threads, it probably needs to live on the
[heap], and because that shared state can't be mutable it needs to be protected by
some synchronization primitive.

[heap]: https://stackoverflow.com/questions/79923/what-and-where-are-the-stack-and-heap

So, _hoist the data inside your implementation of `KvEngine`, `KvsStore` onto
the heap using a thread-safe shared pointer type and protect it behind a lock of
your choosing_.

Remember that because `KvsEngine` requires `Clone` you also need to implement
`Clone` for `KvsStore`.

At this point, if you've carried over your code from previous projects, your
single-threaded `KvServer` should work once again, but now with a `KvsEngine`
that can later be shared across threads.

**Test cases to complete**:

  - TODO @Yilin


## Part 3: Adding multithreading to `KvServer`

Let's quickly review our architecture here: `KvServer` sets up a TCP socket and
begins listening on it; when it receives a request it deserializes it and calls
some implementation of the `KvsEngine` trait to store or retrieve data; then it
sends back the response. The details of how `KvsEngine` works don't matter to
`KvServer`.

So in the last project you probably created a loop vaguelly like:

```rust
let listener = TcpListener::bind(addr)?;

for stream in listener.incoming() {
	let cmd = self.read_cmd(&stream);
	let resp = self.process_cmd(cmd);
	self.respond(&stream, resp);
}
```

_Well, now you just need to do the same thing, but spawn all the work inside the
loop into your `NaiveThreadPool`_. The database query and the response are both
handled on a different thread than the TCP listener. This offloads most of
the hard work to other threads, allowing the recieving thread to process more
requests. It should increase throughput, at least on multi-core machines.

**Test cases to complete**:

  - TODO @Yilin

Again, you should still have a working client/server key-value store, now
multithreaded.


## Part 4: Creating a real thread pool

So now that you've got your multi-threaded architecture in place, it's time to
write a real thread pool. You probably wouldn't write your own thread pool in
practice as there exist thread pool crates that are well-tested, but it
is a useful exercise to gain experience with parallelism in general. Later in
this project you will, as we did with the engine in the previous project,
abstract the thread pool and compare the performance of yours with an existing.

So, what is a thread pool?

It's nothing complicated. Instead of creating a new thread for every
multi-threaded job to be performed, a thread pool maintains a "pool" of threads,
and reuses those threads instead of creating a new one.

But why?

It's entirely about performance. Reusing threads saves a small amount of
performance, but when writing high-performance applications, every bit counts.
Imagine what it takes to make a new thread:

You've got to have a call stack for that thread to run on. That call stack must
be allocated. Allocations are pretty cheap, but not as cheap as no allocation.
How that call stack is allocated depends on details of the operating system and
runtime, but can involve locks and syscalls. Syscalls again are not _that_
expensive, but they are expensive when we're dealing with Rust levels of
performance &mdash; reducing syscalls is a common source of easy optimizations.
That stack then has to be carefully initialized so that first [stack _frame_]
contains the appropriate values for the return pointer and other things. In Rust
the stack needs to be configured with a [guard page] to prevent stack overflows,
preserving memory safety. That takes two more syscalls, [to `mmap` and to
`mprotect`][mp] (though on Linux in particular, those two syscalls are avoided).

[2mb]: https://github.com/rust-lang/rust/blob/6635fbed4ca8c65822f99e994735bd1877fb063e/src/libstd/sys/unix/thread.rs#L12
[mp]: https://github.com/rust-lang/rust/blob/6635fbed4ca8c65822f99e994735bd1877fb063e/src/libstd/sys/unix/thread.rs#L315

TODO: illustration?

That's just setting up the callstack. It's at least another syscall to create
the new thread, at which point the kernel must do its own internal accounting
for the new thread.

In Rust, the C [libpthread] library handles most of this complexity.

Then at some point the OS performs a [context switch] onto the new thread. When
the thread terminates all that work needs to be undone again.

With a thread pool, all that setup overhead is only done for a few threads, and
subsequent jobs are simply context switches into existing threads in the pool.

### So how do you build a thread pool?

There are many strategies and tradeoffs, but for this exercise you are going to
use a single shared queue to distribute work to idle threads. That mans that
your "producer", the thread that recieves network connections, sends jobs to a
single queue (or channel), and every idle thread in the pool reads from that
channel waiting for a job to execute. This is the very simplest work scheduling
strategy, but it can be effective. What are the downsides?

You have three important considerations here:

1) _which data structure to use to distribute the work_ &mdash; it's going to be a
  queue, and there is going to be one sender ("producer"), the thread listening
  for TCP connections, and many recievers ("consumers"), the threads in the pool.

2) _how to deal with panicking jobs_ &mdash; your pool runs arbitrary work items.
  If a thread panics, the thread pool needs to recover in some way.

3) _how to deal with shutdown_ &mdash; when the `ThreadPool` object goes out of
  scope it needs to shut down every thread. It must not leave them idle.

These concerns are all intertwined since dealing with each of them may involve
communication and synchronization between threads. Some solutions will be
simple, the solutions to each of these working together gracefully; some
solutions will be complex, the solutions being independent and convoluted.
Choose your data structures carefully and use their capabilities wisely.

You will distribute work by sending messages over some concurrent queue type (a
concurrent queue in Rust typically being a data structure with two connected
types: sender types, and reciever types; and that can send between the two types
any type that implements `Send` + `'static`).

Messages is Rust are typically represented as enums, with variants for each
possible message that can be sent, like:

```
enum ThreadPoolMessage {
    RunJob(Box<FnOnce + Send + 'static>),
	Shutdown,
}
```

This tends to be a simpler and more efficient solution than trying to "juggle"
multiple channels for different purposes. Of course, if there is only one type
of message, an enum is not necessary. Now, the above example may or may not be
the full set of messages you need to manage your thread pool, depending on the
design. In particular, shutting down can often be done implicitly if your queue
returns a result indicating that the sender has been destroyed.

There are many types of multi-threaded queues. In Rust the most common is
certainly the [`mpsc`] channel, because it lives in Rust's standard library.
This is a multi-producer, single consumer queue, so using it for your
single-queue thread pool will require a lock of some kind. What's the downside
of using a lock here? There are many other concurrent queue types in Rust, and
each has pros and cons. If you are willing to take a lock, then you could even
use a `Mutex<VecDeque>`, but there's probably no reason to do that in production
when better solutions exist.

_Historical note: the existance of channels in Rust's standard library is a bit
of a curiosity, and is widely considered a mistake, as it betrays Rust's general
philosophy of keeping the standard library minimal, focused on abstracting the
operating system, and letting the crate ecosystem experiment with advanced data
structures. Their presence is an artifact of Rust's development history and
origins as a [CSP]-style language like go. Other libraries like [`crossbeam`]
provide more sophisticated alternatives that provide alternative, and (😉)
sometimes more suitable options_.

Your thread pool will need to deal with the case where the spawned function
panics &mdash; simply letting panics destroy the threads in your pool would
quickly deplete its pool of threads. So if a thread in your pool panics you need
to make sure that the total number of threads doesn't decrease. So what should
you do? You have two options: let the thread die and spawn another, or catch the
panic and keep the existing thread running. What are the tradeoffs? You've got
to pick one, but leave a comment in your code explaining your choice.

Some of the tools at your disposal are `[thread::spawn]`, [`catch_unwind`],
[`mpsc`] channels, [`Mutex`], crossbeam's [`mpmc`] channels, and `thread`s
[`JoinHandle`]. Depending on your approach you may use some or all of these.

_Create the `SharedQueueThreadPool` type, implementing `ThreadPool`_.

**Test cases to complete**:

  - `shared_queue_thread_pool_*`

Replace the `NaiveThreadPool` used by `KvServer` with `SharedQueueThreadPool`.
Again your `kvs-server` should still work the same as previously, now with a
slightly more clever multi-threading model. This time you'll want to call
the thread pool constructor with an appropriate number of threads. For
now you can create a thread per CPU, using the [`num_cpus`] crate. We'll
revisit that 


## Part 5: Abstract thread pools

As in the previous project where you created a `KvsEngine` abstraction to compare
different implementations, now you are going to use the `ThreadPool` abstraction
to do the same.

If you haven't already, add a second type parameter to `KvServer` to represent
the `ThreadPool` implementation, the constructor to accept the thread pool
as its second argument, and use that threadpool to distribute the work.

Finally create one more `ThreadPool` implementation, `RayonThreadPool`,
using the `ThreadPool` type from the [`rayon`] crate.


## Part 4: Evaluating our thread pool

TODO: There is going to be a massive amount of context switching with
the way these benchmarks are described. It may be pretty bogus.

Now you are going to write _six_ benchmarks, one write-heavy workload comparing
performance of `SharedQueueThreadPool` with varying numbers of threads, one
read-heavy workload comparing the performance of `SharedQueueThreadPool` with
varying number of threads; two more that use `RayonThreadPool` instead of of
`SharedQueueThreadPool`, and finally, yet two more that use `RayonThreadPool` in
conjunction with `SledKvsEngine`.

So as part of this you will need to make sure the `SledKvsEngine` implementation
you wrote as part of the previous project works again in this multi-threaded
context. It should be trivial as sled can be cloned and sent between threads,
just like your engine.

Hopefully the results will be interesting.

Again you will use [criterion].

What you are attempting to test is the throughput of your server under various
conditions. You will be sending many requests in parallel, waiting for the
responses, then ending.

This will be somewhat complicated by the fact that our `KvsClient` is blocking,
that is, it sends a request then waits for a response. If it was non-blocking,
then you could send _many_ requests without waiting for responses, then collect
the responses later. With the blocking `KvsClient` you will need to send each
request in its own thread in order to saturate the server's capacity.

These are going to be _parameterized_ benchmarks, that is, single benchmarks
that are run multiple times with different parameters. Criterion calls
these [benchmarks with inputs][bi].

[bi]: https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html

A benchmarker like criterion runs a single piece of code many times in a loop,
measuring the time it takes through each loop. As such we want to put only the
code we want to measure in the loop, and leave as much outside of the loop as we
can.

So take this simple example of a criterion benchmark with inputs:

```rust
let c = Criterion::default();
let inputs = &[1, 2, 3, 4, 5];

c.bench_function_over_inputs("example", |b, &&num| {
    b.iter(|| {
        // important measured work goes here
	});
}, inputs);
```

That `iter` calls your closure many times. But since you are going to need to
set up a lot of threads beforehand, that is work that you don't want to measure.
If you can do the setup only once for multiple iterations then the setup can
go outside the closure, like

```rust
let c = Criterion::default();
let inputs = &[1, 2, 3, 4, 5];

c.bench_function_over_inputs("example", |b, &&num| {
    // do setup here
    b.iter(|| {
        // important measured work goes here
	});
}, inputs);
```

The code inside the `b.iter` closure is what is measured, setup goes before.
Since you are going to need many threads for your clients, and you only have the
opportunities to create those threads once, prior to the many iterations of your
loop, you'll need to set up a bunch of reusable threads before you iterate over
the benchmark. Fortunately you have the perfect tool for that in your
`SharedQueueThreadPool`. Set that up with a thread per request, and pair it with
some channels to send the request and report that the response is received, and
you will have a suitable benchmark harness.

**Ok, now to the first two benchmarks**.

We have said that this is a parameterized benchmark, and the parameter to the
benchmark is the number of CPUs to use in the server's thread pool. We want to
see what the throughput is like with just 1 thread, with 2, with 4, and then for
every even number up to 2x the number of cores in your computer. Why 2x? Well,
there may be benefits to having more threads than cores, and you are going to
find out experimentally.

For the write-heavy workload, during setup (the part that runs before the call
to `b.iter(...)`), create the `KvServer<KvStore, SharedQueueThreadPool>`, with
the thread pool containing the parameterized number of threads, and create your
client thread pool containing 1000 threads. Inside the loop, spawn 1000 jobs
that each create a `KvsClient`, and call `set`. The first thread should call
`set("0".to_string(), "0".to_string())`, the second with "00"/"00", and so on.
Then after they have called `set` they should first `assert!` that the call was
successful (to ensure there are no bugs under load), then send back a message to
the benchmarking thread to indicate they are done. When the benchmarking thread
has recieved acknowledgement from all threads, it continues and finishes that
iteration.

Call this benchmark `write_rr_kvstore` (or whatever).

For the read-heavy workload, during setup, create the `KvServer<KvStore,
SharedQueueThreadPool>`, with the thread pool containing the parameterized number
of threads, and create your client thread pool containing 1000 threads. Still in
the setup phase, create yet another client and initialize 1000 key / value
pairs, with key and value equal to "0"/"0" for the first, "00"/"00" the second,
and so on. Then, during the benchmarking loop, from the client, spawn jobs that
issue the same 0-999 `get` requests, and then `assert!` that the result is
correct. Finally, as before, send a message back to the benchmarking thread
indicating that read is complete.

**Whew. That was a lot of work**.

- read-centric workload, measuring network throughput
- is just writing more criterion benchmarks useful?
- can we come up with other tools to measure with,
  or other reasons to measure?
- use parameterized criterion to measure up to 2x
  the number of cores
- use a naive round-robin crate and a work-stealing threadpool (threadpool and rayon::ThreadPool?)

## Part 5: Evaluating other thread pools and engines

## Part : Lock-free shared data structures

## Extension 2: Background compaction

- discuss issues with files and concurrency
- move compaction to a background thread
- need to refactor previous projects to use multiple logs
- this should be fairly challenging



<!--

## Background reading ideas

- scheduling strategies
- shared mutable state, especially in multithreaded context
- threadpools
- something about parallelism and concurrency
- something that explains Arc<Mutex>
- something about the distinction between interior and
  exterior mutability, bonus if it includes parallelism

## TODOs

- a concurrent map or skiplist would be better than a mutexed hashmap but there
  doesn't seem to be a prod-quality crate for it
- is there some new kind of measurement we can do
  for thread pools in addition to criterion benchmarks?
- panic handling for threads in the threadpool

-->
