package main

// MergeSort performs the merge sort algorithm.
// Please supplement this function to accomplish the home work.
//func main() {
//	input := []int64{0,1,5,3,4,2}
//	MergeSort(input)
//	fmt.Println(input)
//}

func MergeSort(src []int64) {
	ch := make(chan int, 1)
	defer close(ch)
	process(src, 0, len(src)-1, ch)
}

func process(src []int64, left int, right int, c chan int) {
	if left < right {
		ch := make(chan int, 2)
		defer close(ch)
		mid := left + (right - left) >>1
		// fmt.Println(mid)
		go process(src, left, mid,ch)
		go process(src, mid+1, right,ch)
		<- ch
		<- ch
		merge(src, left, mid, right)
	}
	c <- 1
}

func merge(src []int64, left int, mid int, right int) {
	help := make([]int64, right-left+1)
	p1, p2 := left, mid+1
	index := 0
	for ; p1 <= mid && p2 <= right; {
		if src[p1] < src[p2] {
			help[index] = src[p1]
			index ++
			p1++
		} else {
			help[index] = src[p2]
			index ++
			p2 ++
		}
	}
	for ; p1 <= mid; {
		help[index] = src[p1]
		index ++
		p1 ++
	}
	for ; p2 <= right; {
		help[index] = src[p2]
		index++
		p2 ++
	}
	for j := 0; j < len(help); j++ {
		src[left+j] = help[j]
	}
}
