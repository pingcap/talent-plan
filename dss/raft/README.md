# The Raft lab

This is a series of labs on a key/value storage system built with the Raft
consensus algorithm. These labs are derived from the [lab2:raft][6824lab2] and
[lab3:kvraft][6824lab3] from the famous [MIT 6.824][6824] course but rewritten
in Rust. The following text material is also very influenced by the text
material there.

[6824lab2]:http://nil.csail.mit.edu/6.824/2018/labs/lab-raft.html
[6824lab3]:http://nil.csail.mit.edu/6.824/2018/labs/lab-kvraft.html
[6824]:http://nil.csail.mit.edu/6.824/2018/index.html

In these labs your will first implement the raft consensus algorithm in the
lab2:raft, and then build a key/value service in the lab3:kvraft.

Raft is a consensus algorithm that is designed to be easy to understand.You can
read material about the Raft itself at [the Raft site][raftsite], including
[the extended Raft paper][raftpaper], an interactive visualization of the Raft,
and other resource. Those material should be helpful for you to complete this lab.

[raftsite]:https://raft.github.io/

## Getting started

First, please clone this repository with `git` to get the source code of the labs.

Then, make sure you have `rustup` install, and override toolchain for this folder
to nightly with `rustup override set nightly` . Also, to make things simpler you
should have `make` installed.

Now you can run `make test_others` to check that things are going right. You
should see all tests passed.

(If you are a Windows user, you may have to figure out how to use `make` on
Windows, or type the commands from the makefile in the console manually, or just
use the Windows Subsystem for Linux)

## The lab2: Raft

In this lab you will implement the Raft consensus algorithm. This lab has 3 parts
named with 2A, 2B and 2C.

To run all the test in this lab, run `make test_2`. Please run the tests multiple
times to mak sure your are not passing the tests just by luck.

To run just a single test, run `make cargo_test_<insert test name here>`.

### The code structure

All your codes in this lab should be in the `src/raft/service.rs` and
`src/raft/mod.rs`.

The `src/raft/mod.rs` file should contains your main implementation of Raft.
The tester (and your key/value server in lab3) will call methods in this file
to use your Raft module.

A service calls `Raft::new` to create a Raft peer, and then calls `Node::new` to
start the Raft peer. The `Node::get_state`, `Node::is_leader` and `Node::term`
will be called to get the current term of the node and whether it thinks it is
leader.

The `Node::start` will be called when the server need to append the command into
the log. `Node::start` should returns immediately without waiting for the log
appends to complete. A channel (`apply_ch`) is passed into `Raft::new` and you
should send a `ApplyMsg` to the channel for each newly committed log entry.

Your implements should use the provided `labrpc` crate to exchange RPCs. The
`labrpc` use channels to simulate sockets internally. This makes it easy to test
your code under challenging network conditions. Your definition of the RPCs
should in `src/raft/service.rs` and you should implement the RPC server in
`impl RaftService for Node`. A set of RPC clients (`peers`) is pass into
`Raft::new` for you to send RPCs to other nodes.

### Part 2A

In this part you should implement the leader election and heartbeats
(`AppendEntries` RPCs without log entries). You should make a single leader to be
elected, make leader to remain leader when there are no failures, and have a new
leader when old leader fails or packets to/from old leader is lost.

To run all the test in this lab, run `make test_2a`.

Here are some hints on this part:

- Add any state you need to the `Raft` struct.
- The `request_vote` RPC is already defined, you just need to fill the
`RequestVoteArgs` and `RequestVoteReply` struct. The lab use `labcodec` crate to
encode and decode messages in RPC, which internally use the `prost` external
crate. See the [prost document][prost] to know how to define structs that is used
as messages with `#[Derive(Message)]` and `#[prost(...)]`.
- You need to define the `append_entries` RPC by yourself. `labrpc` use a
`labrpc::service!` macro to define RPC service and generate server and client
traits from your definition. There is an example in `labrpc/examples/echo.rs`
which may help you to define new RPCs.
- This lab use things from the `futures` external crate heavily like the channels
and the `Future` trait. Read things about futures [here][futures].
- You need to make your code take actions periodically or after delays in time.
You can use lots of threads and call `std::thread::sleep` ([doc][sleep]), but you
can also use the `futures_timer::Delay` and other utilities from the
`futures-timer` external crate ([doc][futures-timer]).
- Don't forget that you need to make sure that the election timeouts in different
peers don't always fire at the same time, or else all peers will vote only for
themselves and no one will become the leader. You can generate random numbers
using the `rand` external crate ([doc][rand]).
- The tester limits the frequency of RPC calls to 10 times per second for each
pair of sender and receiver. Please do not just send RPCs repeatedly without wait
for a timeout.
- The tester requires your Raft to elect a new leader within five seconds of the
failure of the old leader (if a majority of peers can still communicate). However,
leader election may require multiple rounds in case of a split vote (which can
happen if packets are lost or if candidates unluckily choose the same random
backoff times). You must pick election timeouts (and thus heartbeat intervals)
that are short enough that it's very likely that an election will complete in less
than five seconds even if it requires multiple rounds.
- But also, since the tester limits the frequency of RPC calls, your election
timeout should not be too small, and larger than the 150~300 milliseconds in the
paper's Section 5.2. Choose the values wisely.
- In Rust we lock data instead of code. Think carefully about what data should be
in the same lock.
- The Figure 2 in the [Raft paper][raftpaper] is useful when you are in trouble
passing the test.
- It's useful to debug by printing log messages. We use the external
`log` crate ([doc][log]) to print logs in different levels. you can configure
the log level and scope by set LOG_LEVEL environment variable like
`LOG_LEVEL=labs6824=debug make test_2a`. This feature is provided by the
`env_logger` external crate ([doc][env_logger]), read it's documentation for
syntax of the log level. Also, you can collect the output in a file by redirecting
the output to a file like `make test_2a 2>test.log`

[prost]:https://github.com/danburkert/prost
[futures]:https://docs.rs/futures/0.1/futures/
[sleep]:https://doc.rust-lang.org/std/thread/fn.sleep.html
[futures-timer]:https://docs.rs/futures-timer/0.1/futures_timer/index.html
[rand]:https://docs.rs/rand/0.4/rand/
[log]:https://docs.rs/log/0.4/log/
[env_logger]:https://docs.rs/env_logger/0.6/env_logger/

### Part 2B

In this part you should implement the log replication. You should implement the
`Node::Start` method, complete the rest fields in the `append_entries` RPC and
send them, and advance `commit_index` at leader.

To run all the test in this lab, run `make test_2b`. You can try to pass the
`test_basic_agree_2b` test first.

Here are some hints on this part:

- Don't forget election restriction, see section 5.4.1 in the
[Raft paper][raftpaper].
- Every server commit new entries independently by write to `apply_ch` in the
correct order. `apply_ch` is a `UnboundedSender` which will buffering the messages
until out of memory, so it is not that easy to create deadlocks as the original
go version.
- Give yourself enough time to rewrite your implementation because only after
writing a first implementation will you realize how to organize your code cleanly.
- The `test_count_2b` requires your number of RPCs to be not too much when there
is no failure. So you should optimize the number of RPCs to the minimum.
- You may need to write code that waits for certain events to occur. In that case
you can just use channels and wait on it.

### Part 2C

In this part you should implement persistence by first adding code that saves and
restores persistent state to `Persister`, like in `Raft::persist` and
`Raft::restore` using `labcodec`. You also need to determine what and when to
persist, and call `Raft::restore` in `Raft::new`.

To run all the test in this lab, run `make test_2b`. You can try to pass the
`test_persist1_2c` test first.

Here are some hints on this part:
- The usage of `labcodec` is covered in the hints of part 2A.
- This part also introduce various challenging test that involving servers failing
and the network losing RPC requests or replies. Check your implementation
carefully to find bugs that only present in this situation.
- In order to pass some of the challenging tests towards the end, such as those
marked "unreliable", you will need to implement the optimization to allow a
follower to back up the leader's nextIndex by more than one entry at a time. See
the description in the [Raft paper][raftpaper] starting at the bottom of page 7
and top of page 8 (marked by a gray line). However, the paper do not have many
details on this. You will need to fill in the gaps, perhaps with the help of
[this 6.824 Raft lectures][optimize-hint].

[optimize-hint]:http://nil.csail.mit.edu/6.824/2018/notes/l-raft2.txt

## The lab3: KvRaft

In this lab you will build a fault-tolerant key-value storage service using the
Raft module in lab 2. This lab has 2 parts named with 3A and 3B.

To run all the test in this lab, run `make test_3`. Please run the tests multiple
times to mak sure your are not passing the tests just by luck.

To run just a single test, run `make cargo_test_<insert test name here>`.

### The code structure

All your codes in this lab should be in the `src/kvraft/service.rs`,
`src/kvraft/server.rs` and `src/kvraft/client.rs`. The file name explains what
they are. Also, you need to modify the files you touched in lab2:raft.

### Part 3A

In this part you should first implement a solution that works when there are no
dropped messages, and no failed servers. Your service must ensure that `get(...)`
and `put_append(...)` are [linearizable][linearizable].

[linearizable]:https://en.wikipedia.org/wiki/Linearizability

That means, completed application calls to the methods on the `Clerk` struct in
`src/kvraft/client.rs` must appear to all clients to have affected the service in
the same linear order, even in there are failures and leader changes. A
`Clerk::Get` that starts after a completed `Clerk::Put` or `Clerk::Append` should
see the value written by the most recent `Clerk::Put` or `Clerk::Append` in the
linear order. Completed calls should have exactly-once semantics.

A reasonable plan of implementation should be:

- Client send RPC request in the `src/kvraft/client.rs`
- Enter the operation from client into the Raft log by `raft::Node::start` in the
RPC handler of `KvServer`
- Execute the operation when the raft log committed, and then reply to RPC

After implement that you should pass the basic one client test, run
`make cargo_test_basic_3a` to check it.

Here are some hints on this part:

- You should receive commit message from Raft by `apply_ch` at the same time you
receives RPC.
- If a leader has called `Raft::start` for a Clerk's RPC, but loses its leadership
before the request is committed to the log, you should make the client to re-send
the RPC request to other servers until it finds the new leader. You can detect
this by check things received from `apply_ch`.
- The `Clerk` client should remember who is the last leader, and try the last
leader first. This will avoid wasting time searching for the leader on every RPC.
- The server should not complete a `get` RPC if it is not part of a majority and
do not has up-to-date data. You can just put the get operation into the log, or
implement the optimization for read-only operations that is described in Section 8
in the [Raft paper][raftpaper].
- Think how to use lock at the beginning.

Then, you should deal with duplicate client requests, including situations where
the client sends a request to a server leader in one term, times out waiting for a
reply, and re-sends the request to a new leader in another term. The request
should always execute just once.

After this you should pass all tests in this part. To run all the test in
this lab, run `make test_3a`.

Here are some hints on this part:

- You will need to uniquely identify client operations to ensure that the key/
value service executes each one just once.
- Your scheme for duplicate detection should free server memory quickly, like
using a hashtable for and only for uncommitted logs.

### Part 3B

In your current implementation, a rebooting server replays the complete Raft log
in order to restore its state. However, it's not practical for a long-running
server to remember the complete Raft log forever.

Instead, you'll modify Raft and kvserver to cooperate to save space: from time to
time kvserver will persistently store a "snapshot" of its current state, and Raft
will discard log entries that precede the snapshot. When a server restarts (or
falls far behind the leader and must catch up), the server first installs a
snapshot and then replays log entries from after the point at which the snapshot
was created. Section 7 of the [Raft paper][raftpaper] outlines the scheme; you
will have to design the details.

The tester passes a `maxraftstate` to the `KvServer::new` indicating the maximum
allowed size of your persistent Raft state in bytes (including the log, but not
including snapshots). You should check the size of Raft state and when Raft state
size is approaching this threshold, it should save a snapshot, and tell the Raft
library that it has snapshotted, so that Raft can discard old log entries.
The `maxraftstate` is a `Option<usize>` and you do not have to snapshot when it
is `None`.

First you should modify the Raft implement to accept a compaction request and
discard entries before the given index, and continue operating while storing only
log entries after that index. The tests from lab2:raft should still pass.

Then you should modify the kvserver so that it can hands snapshots to Raft and
request compaction when Raft state grows too large. The snapshots should be saved
in `raft::Persister`

Here are some hints on this part:

- You are allowed to add methods to your Raft so that kvserver can manage the
process of trimming the Raft log and manage kvserver snapshots.
- You can test your Raft and kvserver's ability to operate with a trimmed log,
and its ability to re-start from the combination of a kvserver snapshot and
persisted Raft state, by running the Lab 3A tests while overriding `maxraftstate`
to `Some(1)`.
- Think about what should be in the snapshot. You should save new snapshot and
restore latest snapshot with `raft::Persister`.
- Uncommitted logs can also in snapshots, so your kvserver must still be able to
detect duplicated operations under this situation.

After that, you should define the `install_snapshot` RPC and the leader should
send this RPC when the leader has discarded the log entries the follower needs.
When a follower receives an `install_snapshot` RPC, it should send the snapshot to
kvserver (maybe in `apply_ch`).

After this you should pass all tests in this part. To run all the test in
this lab, run `make test_3b`.

Here are some hints on this part:

- You do not need to send the snapshots in multiple RPCs. Just send the entire
snapshot in a single `install_snapshot` RPC and that should be enough for this lab.
- You can try `test_snapshot_rpc_3b` first

[raftpaper]:https://raft.github.io/raft.pdf
