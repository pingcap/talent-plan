package main

import (
"fmt"
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
func Merge(src []int64, lowLeft, highLeft int, lowRight, highRight int , wait *sync.WaitGroup){
	defer wait.Done()
	var result []int64
	leftPtr, rightPtr := lowLeft,lowRight;
	for leftPtr < highLeft && rightPtr < highRight {
		if(src[leftPtr] < src[rightPtr]) {
			result = append(result, src[leftPtr])
			leftPtr ++
		} else {
			result = append(result, src[rightPtr])
			rightPtr ++
		}
	}
	result = append(result, src[leftPtr:highLeft]...)
	result = append(result, src[rightPtr:highRight]...)
	for i := lowLeft; i < highRight; i++ {
		src[i] = result[i-lowLeft]
	}
}

func MergeSort(src []int64){
	totalLength := len(src)
	var wait sync.WaitGroup
	for length := 1; length < totalLength; length *= 2 {
		total := (totalLength + length*2 - 1) / ( length * 2 )
		wait.Add(total)
		for current := 0; current < total; current++ {
			lowLeft := 2*current*length
			highLeft := minInt(lowLeft + length,totalLength)
			lowRight := highLeft
			highRight := minInt(lowRight + length,totalLength)
			go Merge(src, lowLeft, highLeft, lowRight, highRight, &wait)
		}
		wait.Wait()
	}
}

func main() {
	arr := []int64{321,1,2,222,7,8,3,444,555};
	fmt.Println(arr);
	MergeSort(arr);
	fmt.Println(arr);
}