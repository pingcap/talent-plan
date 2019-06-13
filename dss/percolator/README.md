# The Percolator lab

## What is Percolator

Percolator is a system built by Google for incremental processing on a very
large data set. Percolator also provides a distributed transaction protocol with
ACID snapshot-isolation semantics. You can find more details in the paper:
[Large-scale Incremental Processing Using Distributed Transactions and Notifications](https://storage.googleapis.com/pub-tools-public-publication-data/pdf/36726.pdf).

## Lab prerequisites

To start this lab, there are some prerequisites you need to:

1. be familiar with Rust (You can also learn something from our Rust training
course)

2. know about how protobuf works

3. have basic knowledge of how RPC works

4. have basic knowledge of what is the distributed transaction

## Concepts of the lab

### Server

There are two kinds of servers which provide different services in this lab: the
TSO server and the storage server.

#### TSO server

Percolator relies on a service named *timestamp oracle*. The TSO server
implemented by `TimestampOracle` can produce timestamps in a strictly increasing
order. All transactions need to get the unique timestamp to indicate the
execution order.

#### Storage server

Percolator is built upon the Bigtable, which presents a multi-dimensional sorted
map to users. In this lab, the storage server implemented by `MemoryStorage`,
which consists of three columns, is used to simulate the Bigtable. These columns
which implemented by `BTreeMap` are similar to the column in the Bigtable. In
particular, the `MemoryStorage` has three columns: `Write`, `Data`, `Lock` to
keep consistent with the Bigtable.

Besides, the storage also needs to provide the basic operations like `read`,
`write` and `erase` to manipulate the data stored in it.

### Client

The client will `begin` a transaction which contains a set of operations, like
`get` and `set`, and call `commit` to commit a transaction. Also, the client
will call `get_timestamp` to obtain a timestamp.

More implementation details can be found in the paper.

## Writing your own implementation

There are some comments leaving in this project such as "Your definitions here"
or "Your code here". You need to write the code by yourself according to the paper.
There are not many strict restrictions, and thus you can define as many variables
in both *struct* and *proto* as required to implement the functionality.

## Testing your work

You can directly run the following command in the current directory:

```sh
make test_percolator
```
