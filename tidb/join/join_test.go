package main

import (
	"testing"

	"github.com/pingcap/check"
)

type joinTestSuite struct{}

func TestT(t *testing.T) {
	check.TestingT(t)
}

var _ = check.Suite(&joinTestSuite{})

func (s *joinTestSuite) TestJoin(c *check.C) {
	for _, t := range []struct {
		f0       string
		f1       string
		offsets0 []int
		offsets1 []int
		sum      uint64
	}{
		// r0 join r0 on r0.col0 = r0.col1
		{"./t/r0.tbl", "./t/r0.tbl", []int{0}, []int{1}, 0x16CBF2D},
		// r0 join r1 on r0.col0 = r1.col0
		{"./t/r0.tbl", "./t/r1.tbl", []int{0}, []int{0}, 0xC1D73B},
		// r0 join r2 on r0.col0 = r2.col0
		{"./t/r0.tbl", "./t/r2.tbl", []int{0}, []int{0}, 0x1F235},
		// r0 join r1 on r0.col0 = r1.col0 and r0.col1 = r1.col1
		{"./t/r0.tbl", "./t/r1.tbl", []int{0, 1}, []int{0, 1}, 0},
		// r1 join r2 on r1.col0 = r2.col0
		{"./t/r1.tbl", "./t/r2.tbl", []int{0}, []int{0}, 0x18CDA},
		// r2 join r2 on r2.col0 = r2.col0 and r2.col1 = r2.col1
		{"./t/r2.tbl", "./t/r2.tbl", []int{0, 1}, []int{0, 1}, 0x5B385},
	} {
		c.Assert(Join(t.f0, t.f1, t.offsets0, t.offsets1), check.Equals, t.sum)
		//c.Assert(JoinExample(t.f0, t.f1, t.offsets0, t.offsets1), check.Equals, t.sum)
	}
}
