package main

import (
	"fmt"
	"math/rand"
	"path"
	"sort"
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
		return fmt.Sprintf("%dKB", d/KB)
	} else if d < GB {
		return fmt.Sprintf("%dMB", d/MB)
	}
	return fmt.Sprintf("%dGB", d/GB)
}

// Case represents a test case.
type Case struct {
	MapFiles   []string // input files for map function
	ResultFile string   // expected result
}

// CaseGenF represents test case generate function
type CaseGenF func(dataFileDir string, totalDataSize, nMapFiles int) Case

// AllCaseGenFs returns all CaseGenFs used to test.
func AllCaseGenFs() []CaseGenF {
	var gs []CaseGenF
	gs = append(gs, genUniformCases()...)
	gs = append(gs, genPercentCases()...)
	gs = append(gs, CaseSingleURLPerFile)
	return gs
}

func genUniformCases() []CaseGenF {
	cardinalities := []int{1, 7, 200, 10000, 1000000}
	gs := make([]CaseGenF, 0, len(cardinalities))
	for i := range cardinalities {
		card := cardinalities[i]
		gs = append(gs, func(dataFileDir string, totalDataSize, nMapFiles int) Case {
			if FileOrDirExist(dataFileDir) {
				files := make([]string, 0, nMapFiles)
				for i := 0; i < nMapFiles; i++ {
					fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
					files = append(files, fpath)
				}
				rpath := path.Join(dataFileDir, "result")
				return Case{
					MapFiles:   files,
					ResultFile: rpath,
				}
			}
			urls, avgLen := randomNURL(card)
			eachRecords := (totalDataSize / nMapFiles) / avgLen
			files := make([]string, 0, nMapFiles)
			urlCount := make(map[string]int, len(urls))
			for i := 0; i < nMapFiles; i++ {
				fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
				files = append(files, fpath)
				f, buf := CreateFileAndBuf(fpath)
				for i := 0; i < eachRecords; i++ {
					str := urls[rand.Int()%len(urls)]
					urlCount[str]++
					WriteToBuf(buf, str, "\n")
				}
				SafeClose(f, buf)
			}

			rpath := path.Join(dataFileDir, "result")
			genResult(rpath, urlCount)
			return Case{
				MapFiles:   files,
				ResultFile: rpath,
			}
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
	for i := range ps {
		p := ps[i]
		gs = append(gs, func(dataFileDir string, totalDataSize, nMapFiles int) Case {
			if FileOrDirExist(dataFileDir) {
				files := make([]string, 0, nMapFiles)
				for i := 0; i < nMapFiles; i++ {
					fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
					files = append(files, fpath)
				}
				rpath := path.Join(dataFileDir, "result")
				return Case{
					MapFiles:   files,
					ResultFile: rpath,
				}
			}

			// make up percents list
			percents := make([]float64, 0, p.l)
			percents = append(percents, p.p...)
			var sum float64
			for _, p := range p.p {
				sum += p
			}
			if sum > 1 || len(p.p) > p.l {
				panic("invalid prefix")
			}
			x := (1 - sum) / float64(p.l-len(p.p))
			for i := 0; i < p.l-len(p.p); i++ {
				percents = append(percents, x)
			}

			// generate data
			urls, avgLen := randomNURL(len(percents))
			eachRecords := (totalDataSize / nMapFiles) / avgLen
			files := make([]string, 0, nMapFiles)
			urlCount := make(map[string]int, len(urls))

			accumulate := make([]float64, len(percents)+1)
			accumulate[0] = 0
			for i := range percents {
				accumulate[i+1] = accumulate[i] + percents[i]
			}

			for i := 0; i < nMapFiles; i++ {
				fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
				files = append(files, fpath)
				f, buf := CreateFileAndBuf(fpath)
				for i := 0; i < eachRecords; i++ {
					x := rand.Float64()
					idx := sort.SearchFloat64s(accumulate, x)
					if idx != 0 {
						idx--
					}
					str := urls[idx]
					urlCount[str]++
					WriteToBuf(buf, str, "\n")
				}
				SafeClose(f, buf)
			}

			rpath := path.Join(dataFileDir, "result")
			genResult(rpath, urlCount)
			return Case{
				MapFiles:   files,
				ResultFile: rpath,
			}
		})
	}
	return gs
}

// CaseSingleURLPerFile .
func CaseSingleURLPerFile(dataFileDir string, totalDataSize, nMapFiles int) Case {
	if FileOrDirExist(dataFileDir) {
		files := make([]string, 0, nMapFiles)
		for i := 0; i < nMapFiles; i++ {
			fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
			files = append(files, fpath)
		}
		rpath := path.Join(dataFileDir, "result")
		return Case{
			MapFiles:   files,
			ResultFile: rpath,
		}
	}
	urls, avgLen := randomNURL(nMapFiles)
	eachRecords := (totalDataSize / nMapFiles) / avgLen
	files := make([]string, 0, nMapFiles)
	urlCount := make(map[string]int, len(urls))
	for i := 0; i < nMapFiles; i++ {
		fpath := path.Join(dataFileDir, fmt.Sprintf("inputMapFile%d", i))
		files = append(files, fpath)
		f, buf := CreateFileAndBuf(fpath)
		for j := 0; j < eachRecords; j++ {
			str := urls[i]
			urlCount[str]++
			WriteToBuf(buf, str, "\n")
		}
		SafeClose(f, buf)
	}

	rpath := path.Join(dataFileDir, "result")
	genResult(rpath, urlCount)
	return Case{
		MapFiles:   files,
		ResultFile: rpath,
	}
}

func genResult(rpath string, urlCount map[string]int) {
	us, cs := TopN(urlCount, 10)
	f, buf := CreateFileAndBuf(rpath)
	for i := range us {
		fmt.Fprintf(buf, "%s: %d\n", us[i], cs[i])
	}
	SafeClose(f, buf)
}

func randomNURL(n int) ([]string, int) {
	length := 0
	urls := make([]string, 0, n)
	for i := 0; i < n; i++ {
		url := wrapLikeURL(fmt.Sprintf("%d", i))
		length += len(url)
		urls = append(urls, url)
	}
	return urls, length / len(urls)
}

var urlPrefixes = []string{
	"github.com/pingcap/tidb/issues",
	"github.com/pingcap/tidb/pull",
	"github.com/pingcap/tidb",
}

func wrapLikeURL(suffix string) string {
	return path.Join(urlPrefixes[rand.Intn(len(urlPrefixes))], suffix)
}
