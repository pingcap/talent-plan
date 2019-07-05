# PingCAP Talent Plan

This is a series of training courses about writing distributed systems in Go and
Rust. It is maintained by PingCAP for training and/or evaluating students, new
employees, and new contributors to [TiDB] and [TiKV]. As such, the courses focus
on subjects relevant to those projects. They are though appropriate for all Go
and Rust programmers &mdash; they do not require any knowledge of or interest in
either TiDB or TiKV.

The courses primarily consist of projects (or "labs") where coding problems are
presented, along with a partial implementation or API description, and a test
suite.

Each course is developed independently, so they vary in their presentation and
their expectations from course-takers. See the individual course documentation
for details.

[TiDB]: https://github.com/pingcap/tidb
[TiKV]: https://github.com/tikv/tikv


## Training courses

- **[Practical Networked Applications in Rust][rust]**. A series of projects
  that incrementally develop a single Rust project from the ground up into a
  high-performance, networked, parallel and asynchronous key/value store. Along
  the way various real-world and practical Rust development subject matter are
  explored and discussed.

- **[Distributed Systems in Rust][dss]**. Adapted from the [MIT 6.824]
  distributed systems coursework, this course focuses on implementing important
  distributed algorithms, including the [Raft] consensus algorithm, and the
  [Percolator] distributed transaction protocol.

- **[Distributed Systems in Go][go]**. A course on implementing algorithms
  necessary in distributed databases, including map reduce, and parallel query
  optimization.

[rust]: ./rust/
[dss]: ./dss/
[go]: ./tidb/

[MIT 6.824]: http://nil.csail.mit.edu/6.824/2017/index.html
[Raft]: https://raft.github.io/
[Percolator]: https://storage.googleapis.com/pub-tools-public-publication-data/pdf/36726.pdf

## License

These courses may be freely used and modified for any purpose, under the terms
of each course's individual license. See the courses for details.
