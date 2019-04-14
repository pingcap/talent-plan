package main

import (
	"bufio"
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"sort"
	"strings"
)

// RoundArgs contains arguments used in a map-reduce round.
type RoundArgs struct {
	MapFunc    MapF
	ReduceFunc ReduceF
	NReduce    int
}

// RoundsArgs represents arguments used in multiple map-reduce rounds.
type RoundsArgs []RoundArgs

type urlCount struct {
	url string
	cnt int
}

// TopN returns topN urls in the urlCntMap.
func TopN(urlCntMap map[string]int, n int) ([]string, []int) {
	ucs := make([]*urlCount, 0, len(urlCntMap))
	for k, v := range urlCntMap {
		ucs = append(ucs, &urlCount{k, v})
	}
	sort.Slice(ucs, func(i, j int) bool {
		if ucs[i].cnt == ucs[j].cnt {
			return ucs[i].url < ucs[j].url
		}
		return ucs[i].cnt > ucs[j].cnt
	})
	urls := make([]string, 0, n)
	cnts := make([]int, 0, n)
	for i, u := range ucs {
		if i == n {
			break
		}
		urls = append(urls, u.url)
		cnts = append(cnts, u.cnt)
	}
	return urls, cnts
}

// CheckFile checks if these two files are same.
func CheckFile(expected, got string) (string, bool) {
	c1, err := ioutil.ReadFile(expected)
	if err != nil {
		panic(err)
	}
	c2, err := ioutil.ReadFile(got)
	if err != nil {
		panic(err)
	}
	s1 := strings.TrimSpace(string(c1))
	s2 := strings.TrimSpace(string(c2))
	if s1 == s2 {
		return "", true
	}

	errMsg := fmt.Sprintf("expected:\n%s\n, but got:\n%s\n", c1, c2)
	return errMsg, false
}

// CreateFileAndBuf opens or creates a specific file for writing.
func CreateFileAndBuf(fpath string) (*os.File, *bufio.Writer) {
	dir := path.Dir(fpath)
	os.MkdirAll(dir, 0777)
	f, err := os.OpenFile(fpath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0666)
	if err != nil {
		panic(err)
	}
	return f, bufio.NewWriterSize(f, 1<<20)
}

// OpenFileAndBuf opens a specific file for reading.
func OpenFileAndBuf(fpath string) (*os.File, *bufio.Reader) {
	f, err := os.OpenFile(fpath, os.O_RDONLY, 0666)
	if err != nil {
		panic(err)
	}
	return f, bufio.NewReader(f)
}

// WriteToBuf write strs to this buffer.
func WriteToBuf(buf *bufio.Writer, strs ...string) {
	for _, str := range strs {
		if _, err := buf.WriteString(str); err != nil {
			panic(err)
		}
	}
}

// SafeClose flushes this buffer and closes this file.
func SafeClose(f *os.File, buf *bufio.Writer) {
	if buf != nil {
		if err := buf.Flush(); err != nil {
			panic(err)
		}
	}
	if err := f.Close(); err != nil {
		panic(err)
	}
}

// FileOrDirExist tests if this file or dir exist in a simple way.
func FileOrDirExist(p string) bool {
	_, err := os.Stat(p)
	return err == nil
}
