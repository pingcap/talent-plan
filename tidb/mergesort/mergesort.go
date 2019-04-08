package main

import (
	"sort"
	"sync"
)

// MergeSort performs the merge sort algorithm.
// Please supplement this function to accomplish the home work.
func MergeSort(src []int64) {
	if len(src) < 2048 {
		sort.Slice(src, func(i, j int) bool { return src[i] < src[j] })
		return
	}

	mid := len(src) / 2
	wg := sync.WaitGroup{}

	wg.Add(1)
	go func() {
		defer wg.Done()
		MergeSort(src[mid:])
	}()
	MergeSort(src[:mid])
	wg.Wait()

	merge(src, mid)
}

func merge(src []int64, mid int) {
	lhs := make([]int64, mid)
	copy(lhs, src[:mid])
	rhs := src[mid:]

	i, j, k := 0, 0, 0
	for ; i < len(lhs) && j < len(rhs); k++ {
		if lhs[i] <= rhs[j] {
			src[k] = lhs[i]
			i++
		} else {
			src[k] = rhs[j]
			j++
		}
	}

	if i < len(lhs) {
		copy(src[k:], lhs[i:])
	} else if j < len(rhs) {
		copy(src[k:], rhs[j:])
	}
}
