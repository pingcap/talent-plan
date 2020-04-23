## Introduction

This is the homework for PingCAP Talent Plan Online of week 4. This homework is a simplified version of [ACM SIGMOD Programming Contest 2018](http://sigmod18contest.db.in.tum.de/index.shtml).

The task is to evaluate batches of join queries on a set of pre-defined relations. Each join query specifies two relations, multiple (equality) join predicates, and one (sum) aggregation. The challenge is to fully utilize the CPU and memory resources and execute the queries as fast as possible.

NOTE: **go 1.12 is required**

## Details

The simple interface `Join(f0, f1 string, offset0, offset1 []int) (sum uint64)` is defined in `join.go`. Our test harness will feed two relations and two columns' offsets array to the interface every time, and check the correctness of the output result. Explaination to the four input arguments and one output argument of the interface are list as follows:

- **f0**: File name of the given relation0.
- **f1**: File name of the given relation1.
- **offset0**: Offsets of which columns the given relation0 should be joined.
- **offset1**: Offsets of which columns the given relation1 should be joined.
- **sum** (output argument): Sum of the relation0.col0 in the final join result.

The (equality) join predicates are specified by the `offset0/1`. The form of the join predicates is like:
``` go
relation0.cols[offset[0]] = relation1.cols[offset[0]] and relation0.cols[offset[1]] = relation1.cols[offset[1]]...
```

**Example**: `Join("/path/T0", "/path/T1", []int{0, 1}, []int{2, 3})`

Translated to SQL:

``` sql
SELECT SUM(T0.COL0)
FROM T0, T1
ON T0.COL0=T1.COL2 AND T0.COL1=T1.COL3
```

We provide a sample as `join_example.go: JoinExample` which performs a simple hash join algorithm. It uses the relation0 to build the hash table, and probe the hash table for every row in relation1.

## Requirements and rating principles

- (30%) Pass all test cases.
- (20%) Perform better than `join_example.go:JoinExample`.
- (35%) Have a document to describe your idea and record the process of performance optimization with `pprof`.
- (15%) Keep a good code style.

Note:
1. For your check sums, you do not have to worry about numeric overflows as long as you are using 64 bit unsigned integers.
2. More large datasets are provided [here](https://drive.google.com/drive/u/1/folders/10-iJNGKmKXgMmvBYnKt88RTwC0iA1XM-), you can use them to help profile your program.
3. We'll use the `BenchmarkJoin` and `BenchmarkJoinExample` which can be found in `benchmark_test.go` to evaluate your program. Test data will **NOT** be outside of what we've provided.

## How to use

1. Please implement your own `Join` in join.go to accomplish this task.
2. We provide CSV versions (.tbl files) of three relations in directory `t`. You can load them into a DBMS to test your program.
   1. **r0.tbl**: 2 columns * 10,000 records
   2. **r1.tbl**: 4 columns * 5,000 records
   3. **r2.tbl**: 4 columns * 500 records
3. There is already a built-in unit test `JoinTest` defined in `join_test.go`. You can write your own unit tests, but please make sure `JoinTest` can be passed before.
4. Use `make test` to run all the unit tests.
5. Use `make bench` to run all the benchmarks.
