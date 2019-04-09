package main

import (
	"bufio"
	"fmt"
	"math/rand"
	"sort"
	"strconv"
)

type DataSize int

const (
	Byte = 1
	KB   = 1 << 10
	MB   = 1 << 20
	GB   = 1 << 30
)

func (d DataSize) String() string {
	if d < KB {
		return fmt.Sprintf("%dbyte", d)
	} else if d < MB {
		return fmt.Sprintf("%dKB", d / KB)
	} else if d < GB {
		return fmt.Sprintf("%dMB", d / MB)
	}
	return fmt.Sprintf("%dGB", d / GB)
}

// Case represents a test case.
type Case struct {
	MapFiles   []MemFile // input files for map function
	ResultFile MemFile   // expected result
}

// CaseGenF represents test case generate function
type CaseGenF func(totalDataSize, nMapFiles int) Case

// AllCaseGenFs returns all CaseGenFs used to test.
func AllCaseGenFs() []CaseGenF {
	var gs []CaseGenF
	gs = append(gs, genUniformCases()...)
	gs = append(gs, genPercentCases()...)
	gs = append(gs, CaseSingleURLPerFile, CaseNoSameURL)
	return gs
}

func genUniformCases() []CaseGenF {
	cardinalities := []int{11, 200, 10000, 200000, 1000000, 2, 3, 7, 9}
	gs := make([]CaseGenF, 0, len(cardinalities))
	for _, card := range cardinalities {
		gs = append(gs, func(totalDataSize, nMapFiles int) Case {
			return uniformGen(totalDataSize, nMapFiles, card)
		})
	}
	return gs
}

func genPercentCases() []CaseGenF {
	ps := []struct {
		l int
		p []float64
	}{
		{11, []float64{0.9, 0.09, 0.009, 0.0009, 0.00009, 0.000009}},
		{10000, []float64{0.9, 0.09, 0.009, 0.0009, 0.00009, 0.000009}},
		{100000, []float64{0.9, 0.09, 0.009, 0.0009, 0.00009, 0.000009}},
		{10000, []float64{0.5, 0.4}},
		{10000, []float64{0.3, 0.3, 0.3}},
	}
	gs := make([]CaseGenF, 0, len(ps))
	for _, p := range ps {
		gs = append(gs, func(totalDataSize, nMapFiles int) Case {
			percents := makePercents(p.l, p.p...)
			return percentGen(totalDataSize, nMapFiles, percents)
		})
	}
	return gs
}

// CaseSingleURLPerFile .
func CaseSingleURLPerFile(totalDataSize, nMapFiles int) Case {
	urls, avgLen := randomNURL(nMapFiles)
	eachRecords := (totalDataSize / nMapFiles) / avgLen
	files := make([]MemFile, 0, nMapFiles)
	urlCount := make(map[string]int, len(urls))
	for i := 0; i < nMapFiles; i++ {
		f := CreateMemFile(fmt.Sprintf("%v", i))
		buf := bufio.NewWriter(f)
		for j := 0; j < eachRecords; j++ {
			str := urls[i]
			urlCount[str]++
			if _, err := buf.WriteString(str); err != nil {
				panic(err)
			}
			if err := buf.WriteByte('\n'); err != nil {
				panic(err)
			}
		}
		if err := buf.Flush(); err != nil {
			panic(err)
		}
		files = append(files, f)
	}
	return Case{
		MapFiles:   files,
		ResultFile: genResult(urlCount),
	}
}

// CaseNoSameURL .
func CaseNoSameURL(totalDataSize, nMapFiles int) Case {
	eachFileSize := totalDataSize / nMapFiles
	files := make([]MemFile, 0, nMapFiles)
	urlSig := 0
	urlCount := make(map[string]int, 4096)
	for i := 0; i < nMapFiles; i++ {
		f := CreateMemFile(fmt.Sprintf("%v", i))
		buf := bufio.NewWriter(f)
		fileSize := 0
		for fileSize < eachFileSize {
			str := fmt.Sprintf("%d", urlSig)
			urlSig++
			fileSize += len(str) + 1
			urlCount[str]++
			if _, err := buf.WriteString(str); err != nil {
				panic(err)
			}
			if err := buf.WriteByte('\n'); err != nil {
				panic(err)
			}
		}

		if err := buf.Flush(); err != nil {
			panic(err)
		}
		files = append(files, f)
	}
	return Case{
		MapFiles:   files,
		ResultFile: genResult(urlCount),
	}
}

func makePercents(length int, prefix ...float64) []float64 {
	percents := make([]float64, 0, length)
	percents = append(percents, prefix...)

	var sum float64
	for _, p := range prefix {
		sum += p
	}
	if sum > 1 || len(prefix) > length {
		panic("invalid prefix")
	}

	x := (1 - sum) / float64(length-len(prefix))
	for i := 0; i < length-len(prefix); i++ {
		percents = append(percents, x)
	}
	return percents
}

func percentGen(totalDataSize, nMapFiles int, percents []float64) Case {
	urls, avgLen := randomNURL(len(percents))
	eachRecords := (totalDataSize / nMapFiles) / avgLen
	files := make([]MemFile, 0, nMapFiles)
	urlCount := make(map[string]int, len(urls))

	accumulate := make([]float64, len(percents)+1)
	accumulate[0] = 0
	for i := range percents {
		accumulate[i+1] = accumulate[i] + percents[i]
	}

	for i := 0; i < nMapFiles; i++ {
		f := CreateMemFile(fmt.Sprintf("%v", i))
		buf := bufio.NewWriter(f)

		for i := 0; i < eachRecords; i++ {
			x := rand.Float64()
			idx := sort.SearchFloat64s(accumulate, x)
			if idx != 0 {
				idx--
			}
			str := urls[idx]
			urlCount[str]++
			if _, err := buf.WriteString(str); err != nil {
				panic(err)
			}
			if err := buf.WriteByte('\n'); err != nil {
				panic(err)
			}
		}

		if err := buf.Flush(); err != nil {
			panic(err)
		}
		files = append(files, f)
	}
	return Case{
		MapFiles:   files,
		ResultFile: genResult(urlCount),
	}
}

func uniformGen(totalDataSize, nMapFiles, cardinality int) Case {
	urls, avgLen := randomNURL(cardinality)
	eachRecords := (totalDataSize / nMapFiles) / avgLen
	files := make([]MemFile, 0, nMapFiles)
	urlCount := make(map[string]int, len(urls))
	for i := 0; i < nMapFiles; i++ {
		f := CreateMemFile(fmt.Sprintf("%v", i))
		buf := bufio.NewWriter(f)
		for i := 0; i < eachRecords; i++ {
			str := urls[rand.Int()%len(urls)]
			urlCount[str]++
			if _, err := buf.WriteString(str); err != nil {
				panic(err)
			}
			if err := buf.WriteByte('\n'); err != nil {
				panic(err)
			}
		}
		if err := buf.Flush(); err != nil {
			panic(err)
		}
		files = append(files, f)
	}
	return Case{
		MapFiles:   files,
		ResultFile: genResult(urlCount),
	}
}

func genResult(urlCount map[string]int) MemFile {
	us, cs := TopN(urlCount, 10)
	result := CreateMemFile("result")
	for i := range us {
		fmt.Fprintf(result, "%s: %d\n", us[i], cs[i])
	}
	return result
}

func randomNURL(n int) ([]string, int) {
	m := make(map[string]struct{})
	for len(m) < n {
		m[randomURL()] = struct{}{}
	}
	urls := make([]string, 0, len(m))
	totalLen := 0
	for u := range m {
		urls = append(urls, u)
		totalLen += len(u)
	}
	return urls, totalLen / len(urls)
}

func randomURL() string {
	return strconv.Itoa(rand.Int())
}
