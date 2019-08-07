package main

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// ExampleURLTop10 generates RoundsArgs for getting the 10 most frequent URLs.
// There are two rounds in this approach.
// The first round will do url count.
// The second will sort results generated in the first round and
// get the 10 most frequent URLs.
func ExampleURLTop10(nWorkers int) RoundsArgs {
	var args RoundsArgs
	// round 1: do url count
	args = append(args, RoundArgs{
		MapFunc:    ExampleURLCountMap,
		ReduceFunc: ExampleURLCountReduce,
		NReduce:    nWorkers,
	})
	// round 2: sort and get the 10 most frequent URLs
	args = append(args, RoundArgs{
		MapFunc:    ExampleURLTop10Map,
		ReduceFunc: ExampleURLTop10Reduce,
		NReduce:    1,
	})
	return args
}

// ExampleURLCountMap is the map function in the first round
func ExampleURLCountMap(filename string, contents string) []KeyValue {
	lines := strings.Split(string(contents), "\n")
	kvs := make([]KeyValue, 0, len(lines))
	for _, l := range lines {
		l = strings.TrimSpace(l)
		if len(l) == 0 {
			continue
		}
		kvs = append(kvs, KeyValue{Key: l})
	}
	return kvs
}

// ExampleURLCountReduce is the reduce function in the first round
func ExampleURLCountReduce(key string, values []string) string {
	return fmt.Sprintf("%s %s\n", key, strconv.Itoa(len(values)))
}

// ExampleURLTop10Map is the map function in the second round
func ExampleURLTop10Map(filename string, contents string) []KeyValue {
	lines := strings.Split(contents, "\n")
	kvs := make([]KeyValue, 0, len(lines))
	for _, l := range lines {
		kvs = append(kvs, KeyValue{"", l})
	}
	return kvs
}

// ExampleURLTop10Reduce is the reduce function in the second round
func ExampleURLTop10Reduce(key string, values []string) string {
	cnts := make(map[string]int, len(values))
	for _, v := range values {
		v := strings.TrimSpace(v)
		if len(v) == 0 {
			continue
		}
		tmp := strings.Split(v, " ")
		n, err := strconv.Atoi(tmp[1])
		if err != nil {
			panic(err)
		}
		cnts[tmp[0]] = n
	}

	us, cs := TopN(cnts, 10)
	buf := new(bytes.Buffer)
	for i := range us {
		fmt.Fprintf(buf, "%s: %d\n", us[i], cs[i])
	}
	return buf.String()
}
