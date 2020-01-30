package main

import (
	"github.com/pingcap/tidb/util/mvmap"
	"strconv"
	"sync"
)

// Join accepts a join query of two relations, and returns the sum of
// relation0.col0 in the final result.
// Input arguments:
//   f0: file name of the given relation0
//   f1: file name of the given relation1
//   offset0: offsets of which columns the given relation0 should be joined
//   offset1: offsets of which columns the given relation1 should be joined
// Output arguments:
//   sum: sum of relation0.col0 in the final result
//func Join(f0, f1 string, offset0, offset1 []int) (sum uint64) {
//	tbl0, tbl1 := readCSVFileIntoTbl(f0), readCSVFileIntoTbl(f1)
//	if len(tbl0) < len(tbl1) {
//		hashtable := buildHashTable(tbl0, offset0)
//		for _, row := range tbl1 {
//			rowIDs := probe(hashtable, row, offset1)
//			for _, id := range rowIDs {
//				v, err := strconv.ParseUint(tbl0[id][0], 10, 64)
//				if err != nil {
//					panic("Join panic\n" + err.Error())
//				}
//				sum += v
//			}
//		}
//	}
//	return sum
//}
//

var mu sync.RWMutex

// JoinExample performs a simple hash join algorithm.
func Join(f0, f1 string, offset0, offset1 []int) (sum uint64) {
	tbl0, tbl1 := readCSVFileIntoTbl(f0), readCSVFileIntoTbl(f1)
	hashtable := buildHashTable(tbl0, offset0)
	var wg sync.WaitGroup
	for _, row := range tbl1 {
		wg.Add(1)
		go JoinSum(hashtable, row, tbl0, &sum, offset0, &wg)
	}
	for _, _ = range tbl1 {
		wg.Wait()
	}
	return sum
}

func JoinSum(hashtable *mvmap.MVMap, row []string, tbl0 [][]string, sum *uint64, offset []int, wg *sync.WaitGroup) {
	rowIDs := probe(hashtable, row, offset)
	temp := *sum
	for _, id := range rowIDs {
		v, err := strconv.ParseUint(tbl0[id][0], 10, 64)
		if err != nil {
			panic("JoinExample panic\n" + err.Error())
		}
		temp += v
	}
	mu.Lock()
	*sum = *sum + temp
	defer mu.Unlock()
	wg.Done()
}
