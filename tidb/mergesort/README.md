## Introduction

This is the Merge Sort home work for PingCAP Talent Plan Online of week 1.

There are 16, 000, 000 int64 values stored in an unordered array. Please
supplement the `MergeSort()` function defined in `mergesort.go` to sort this
array.

Requirements and rating principles:
* (30%) Pass the unit test.
* (20%) Performs better than `sort.Slice()`.
* (30%) Profile your program with `pprof`, analyze the performance bottleneck.
* (10%) Have a good code style.
* (10%) Document your idea and code.

NOTE: **go 1.12 is required**

## How to use

Please supplement the `MergeSort()` function defined in `mergesort.go` to accomplish
the home work.

**NOTE**:
1. There is a builtin unit test defined in `mergesort_test.go`, however, you still
   can write your own unit tests.
2. There is a builtin benchmark test defined in `bench_test.go`, you should run
   this benchmark to ensure that your parallel merge sort is fast enough.


How to test:
```
make test
```

How to benchmark:
```
make bench
```
