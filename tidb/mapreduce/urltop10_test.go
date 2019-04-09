package main

import (
	"fmt"
	"log"
	"os"
	"path"
	"runtime"
	"testing"
	"time"
)

func testDataScale() ([]DataSize, []int) {
	dataSize := []DataSize{1 * MB, 10 * MB, 100 * MB, 500 * MB, 1 * GB}
	nMapFiles := []int{5, 10, 20, 40, 60}
	return dataSize, nMapFiles
}

const (
	dataDir = "/tmp/mr_homework"
)

func dataPrefix(i int, ds DataSize, nMap int) string {
	return path.Join(dataDir, fmt.Sprintf("case%d-%s-%d", i, ds, nMap))
}

func TestGenData(t *testing.T) {
	gens := AllCaseGenFs()
	dataSize, nMapFiles := testDataScale()
	for k := range dataSize {
		for i, gen := range gens {
			fmt.Printf("generate data file for cast%d, dataSize=%v, nMap=%v\n", i, dataSize[k], nMapFiles[k])
			prefix := dataPrefix(i, dataSize[k], nMapFiles[k])
			gen(prefix, int(dataSize[k]), nMapFiles[k])
		}
	}
}

func TestCleanData(t *testing.T) {
	if err := os.RemoveAll(dataDir); err != nil {
		log.Fatal(err)
	}
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
			prefix := dataPrefix(i, dataSize[k], nMapFiles[k])
			c := gen(prefix, int(dataSize[k]), nMapFiles[k])

			runtime.GC()

			// run map-reduce rounds
			begin := time.Now()
			inputFiles := c.MapFiles
			for idx, r := range rounds {
				jobName := fmt.Sprintf("Case%d-Round%d", i, idx)
				ch := mr.Submit(jobName, prefix, r.MapFunc, r.ReduceFunc, inputFiles, r.NReduce)
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
