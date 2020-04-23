package main

import (
	"encoding/csv"
	"io"
	"os"
	"strconv"
	"unsafe"

	"github.com/pingcap/tidb/util/mvmap"
)

// JoinExample performs a simple hash join algorithm.
func JoinExample(f0, f1 string, offset0, offset1 []int) (sum uint64) {
	tbl0, tbl1 := readCSVFileIntoTbl(f0), readCSVFileIntoTbl(f1)
	hashtable := buildHashTable(tbl0, offset0)
	for _, row := range tbl1 {
		rowIDs := probe(hashtable, row, offset1)
		for _, id := range rowIDs {
			v, err := strconv.ParseUint(tbl0[id][0], 10, 64)
			if err != nil {
				panic("JoinExample panic\n" + err.Error())
			}
			sum += v
		}
	}
	return sum
}

func readCSVFileIntoTbl(f string) (tbl [][]string) {
	csvFile, err := os.Open(f)
	if err != nil {
		panic("ReadFileIntoTbl " + f + " fail\n" + err.Error())
	}
	defer csvFile.Close()

	csvReader := csv.NewReader(csvFile)
	for {
		row, err := csvReader.Read()
		if err == io.EOF {
			break
		} else if err != nil {
			panic("ReadFileIntoTbl " + f + " fail\n" + err.Error())
		}
		tbl = append(tbl, row)
	}
	return tbl
}

func buildHashTable(data [][]string, offset []int) (hashtable *mvmap.MVMap) {
	var keyBuffer []byte
	valBuffer := make([]byte, 8)
	hashtable = mvmap.NewMVMap()
	for i, row := range data {
		for j, off := range offset {
			if j > 0 {
				keyBuffer = append(keyBuffer, '_')
			}
			keyBuffer = append(keyBuffer, []byte(row[off])...)
		}
		*(*int64)(unsafe.Pointer(&valBuffer[0])) = int64(i)
		hashtable.Put(keyBuffer, valBuffer)
		keyBuffer = keyBuffer[:0]
	}
	return
}

func probe(hashtable *mvmap.MVMap, row []string, offset []int) (rowIDs []int64) {
	var keyHash []byte
	var vals [][]byte
	for i, off := range offset {
		if i > 0 {
			keyHash = append(keyHash, '_')
		}
		keyHash = append(keyHash, []byte(row[off])...)
	}
	vals = hashtable.Get(keyHash, vals)
	for _, val := range vals {
		rowIDs = append(rowIDs, *(*int64)(unsafe.Pointer(&val[0])))
	}
	return rowIDs
}
