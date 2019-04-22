package main

import (
	"sync"
)

// MergeSort performs the merge sort algorithm.
// Please supplement this function to accomplish the home work.
func minInt(x, y int) int {
	if x > y {
		return y
	}
	return x
}

func Merge(src []int64, current, length int, wait *sync.WaitGroup, result []int64) {
	totalLength := len(src)
	lowLeft := 2 * current * length
	highLeft := minInt(lowLeft+length, totalLength)
	lowRight := highLeft
	highRight := minInt(lowRight+length, totalLength)
	leftPtr, rightPtr, resultPtr := lowLeft, lowRight, lowLeft

	for leftPtr < highLeft && rightPtr < highRight {
		if src[leftPtr] < src[rightPtr] {
			result[resultPtr] = src[leftPtr]
			leftPtr++
		} else {
			result[resultPtr] = src[rightPtr]
			rightPtr++
		}
		resultPtr++
	}
	for i := leftPtr; i < highLeft; i++ {
		result[resultPtr] = src[i]
		resultPtr++
	}
	for i := rightPtr; i < highRight; i++ {
		result[resultPtr] = src[i]
		resultPtr++
	}

	for i := lowLeft; i < highRight; i++ {
		src[i] = result[i]
	}
	wait.Done()
}

func InsertSort(src []int64, length int) {
	totalLength := len(src)
	for current := 0; current < totalLength; current += length {
		end := minInt(current+length, totalLength)
		for i := current + 1; i < end; i++ {
			for j := i; j > current && src[j] < src[j-1]; j-- {
				src[j], src[j-1] = src[j-1], src[j]
			}
		}
	}
}

func MergeSort(src []int64) {
	totalLength := len(src)
	block := 64
	result := make([]int64, totalLength)
	var wait sync.WaitGroup
	InsertSort(src, block)
	for length := block; length < totalLength; length *= 2 {
		total := (totalLength + length*2 - 1) / (length * 2)
		wait.Add(total)
		for current := 0; current < total; current++ {
			go Merge(src, current, length, &wait, result)
		}
		wait.Wait()
	}
}
