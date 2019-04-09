package main

import (
	"bytes"
	"fmt"
	"io"
	"sort"
	"strings"
)

// MemFile represents a file in memory.
type MemFile interface {
	io.ReadWriter
	Name() string
	Content() string
}

type memFile struct {
	*bytes.Buffer
	name string
}

// Name returns this file's name.
func (f *memFile) Name() string {
	return f.name
}

// Content returns this file's content.
func (f *memFile) Content() string {
	return f.String()
}

// CreateMemFile creates a MemFile with a specific name.
func CreateMemFile(name string) MemFile {
	return &memFile{
		Buffer: new(bytes.Buffer),
		name:   name,
	}
}

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
func CheckFile(expected, got MemFile) (string, bool) {
	c1, c2 := expected.Content(), got.Content()
	c1 = strings.TrimSpace(c1)
	c2 = strings.TrimSpace(c2)
	if c1 == c2 {
		return "", true
	}

	errMsg := fmt.Sprintf("expected:\n%s\n, but got:\n%s\n", c1, c2)
	return errMsg, false
}
