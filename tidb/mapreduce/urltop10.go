package main

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// URLTop10 .
func URLTop10(nWorkers int) RoundsArgs {
	// YOUR CODE HERE :)
	// And don't forget to document your idea.
	var args RoundsArgs
	args = append(args, RoundArgs{
		MapFunc:    UrlTop10Map,
		ReduceFunc: Round1Reduce,
		NReduce:    nWorkers,
	})

	//Round 2
	args = append(args, RoundArgs{
		MapFunc:    Round2Map,
		ReduceFunc: UrlTop10Reduce,
		NReduce:    1,
	})
	return args
}

func UrlTop10Map(filename string, contents string) []KeyValue {
	urls := strings.Split(contents, "\n")
	res := make([]KeyValue, 0, len(urls))
	for _, url := range urls {
		url = strings.TrimSpace(url)
		if len(url) == 0 {
			continue
		}
		res = append(res, KeyValue{Key: url})
	}
	return res
}

func Round1Reduce(key string, values []string) string {
	return fmt.Sprintf("%s %s\n", key, strconv.Itoa(len(values)))
}

func Round2Map(filename string, contents string) []KeyValue {
	lines := strings.Split(contents, "\n")
	kvs := make([]KeyValue, 0, len(lines))
	for _, l := range lines {
		kvs = append(kvs, KeyValue{"", l})
	}
	return kvs
}

func UrlTop10Reduce(key string, values []string) string {
	cnts := make(map[string]int, len(values))
	for _, kv := range values {
		v := strings.TrimSpace(kv)
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
	str, count := TopN(cnts, 10)
	buf := new(bytes.Buffer)
	for i := range str {
		fmt.Fprintf(buf, "%s: %d\n", str[i], count[i])
	}
	return buf.String()
}
