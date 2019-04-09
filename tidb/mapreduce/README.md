## Introduction

This is the Map-Reduce homework for PingCAP Talent Plan Online of week 2.

There is a simple Map-Reduce framework and you should use it to extract the 10 most frequent URLs from data files.

## Getting familiar with the source

The simple Map-Reduce framework is defined in `mapreduce.go`.

The map and reduce function are defined as same as MIT 6.824 lab 1.
```
type ReduceF func(key string, values []string) string
type MapF func(filename string, contents string) []KeyValue
```

There is an example in `urltop10_example.go` which you can use for reference.

Please implement your own `MapF` and `ReduceF` in `urltop10.go` to accomplish this task.

All data files will be generated in memory at runtime, which is implemented in `casegen.go`.

Each test cases has different data distribution and you should take it into account.

## Requirements and rating principles

* (30%) Pass the unit test.
* (30%) Performs better than `urltop10_example`.
* (20%) Profile your program with `pprof`, analyze the performance bottleneck (both the framework and your own code).
* (10%) Have a good code style.
* (10%) Document your idea and code.

NOTE: **go 1.12 is required**

## How to use

Please implement your own `MapF` and `ReduceF` in `urltop10.go` to accomplish this task.

**NOTE**:
1. There is a builtin unit test defined in `urltop10_test.go`, however, you still can write your own unit tests.

How to run example:
```
make test_example
```

How to run your implementation:
```
make test_homework
```
