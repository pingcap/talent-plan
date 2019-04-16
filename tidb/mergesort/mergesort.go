package main

// MergeSort performs the merge sort algorithm.
// Please supplement this function to accomplish the home work.
func Merge(left, right []int64) []int64 {
	var result []int64
	left_ptr, right_ptr := 0,0;
	for left_ptr < len(left) && right_ptr < len(right) {
		if(left[left_ptr] < right[right_ptr]) {
			result = append(result, left[left_ptr])
			left_ptr ++
		} else {
			result = append(result, right[right_ptr])
			right_ptr ++
		}
	} 
	for left_ptr < len(left) {
		result = append(result, left[left_ptr])
		left_ptr ++
	}
	for right_ptr < len(right) {
		result = append(result, right[right_ptr])
		right_ptr ++
	}
	return result
}
func MergeSortMine(src []int64) []int64 {
	length := len(src)
	if length <= 1 {
		return src
	}
	mid_ptr := length / 2
	left := MergeSortMine(src[:mid_ptr])
	right := MergeSortMine(src[mid_ptr:])
	return Merge(left, right)
}

func MergeSort(src []int64) {
	result := MergeSortMine(src)
	copy(src, result)
}