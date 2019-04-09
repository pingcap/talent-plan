package main

import (
	"fmt"
	"runtime"
	"testing"
	"time"
)

func testDataScale() ([]DataSize, []int) {
	dataSize := []DataSize{5 * MB, 10 * MB, 100 * MB}
	nMapFiles := []int{5, 10, 20}
	return dataSize, nMapFiles
}

func TestExampleURLTop(t *testing.T) {
	rounds := ExampleURLTop10(GetMRCluster().NWorkers())
	testURLTop(t, rounds)
}

func TestURLTop(t *testing.T) {
	rounds := URLTop10(GetMRCluster().NWorkers())
	testURLTop(t, rounds)
}

func testURLTop(t *testing.T, rounds RoundsArgs) {
	if len(rounds) == 0 {
		t.Fatalf("no rounds arguments, please finish your code")
	}
	mr := GetMRCluster()

	// run all cases
	gens := AllCaseGenFs()
	dataSize, nMapFiles := testDataScale()
	for k := range dataSize {
		for i, gen := range gens {
			// generate data
			c := gen(int(dataSize[k]), nMapFiles[k])

			runtime.GC()

			// run map-reduce rounds
			begin := time.Now()
			inputFiles := c.MapFiles
			for idx, r := range rounds {
				jobName := fmt.Sprintf("Case%d-Round%d", i, idx)
				ch := mr.Submit(jobName, r.MapFunc, r.ReduceFunc, inputFiles, r.NReduce)
				inputFiles = <-ch
			}
			cost := time.Since(begin)

			// check result
			if len(inputFiles) != 1 {
				panic("the length of result file list should be 1")
			}
			result := inputFiles[0]

			if errMsg, ok := CheckFile(c.ResultFile, result); !ok {
				t.Fatalf("Case%d FAIL, dataSize=%v, nMapFiles=%v, cost=%v\n%v\n", i, dataSize[k], nMapFiles[k], cost, errMsg)
			} else {
				fmt.Printf("Case%d PASS, dataSize=%v, nMapFiles=%v, cost=%v\n", i, dataSize[k], nMapFiles[k], cost)
			}
		}
	}
}
