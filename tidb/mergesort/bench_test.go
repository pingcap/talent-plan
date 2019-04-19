package main

import (
	"sort"
	"testing"

	"github.com/pingcap/check"
)

var _ = check.Suite(&benchTestSuite{})

func TestBench(t *testing.T) {
	check.TestingT(t)
}

type benchTestSuite struct{}

const numElements = 16 << 20

var (
	src      []int64
	original []int64
)

func (b *benchTestSuite) SetUpSuite(c *check.C) {
	src = make([]int64, numElements)
	original = make([]int64, numElements)
	prepare(original)
}

func (b *benchTestSuite) BenchmarkMergeSort(c *check.C) {
	for i := 0; i < c.N; i++ {
		c.StopTimer()
		copy(src, original)
		c.StartTimer()
		MergeSort(src)
	}
}

func (b *benchTestSuite) BenchmarkNormalSort(c *check.C) {
	for i := 0; i < c.N; i++ {
		c.StopTimer()
		copy(src, original)
		c.StartTimer()
		sort.Slice(src, func(i, j int) bool { return src[i] < src[j] })
	}
}
