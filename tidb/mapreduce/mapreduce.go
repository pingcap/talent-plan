package main

import (
	"bufio"
	"encoding/json"
	"hash/fnv"
	"io"
	"log"
	"runtime"
	"sort"
	"strconv"
	"sync"
)

// KeyValue is a type used to hold the key/value pairs passed to the map and reduce functions.
type KeyValue struct {
	Key   string
	Value string
}

// Reduce function from MIT 6.824 LAB1
type ReduceF func(key string, values []string) string

// Map function from MIT 6.824 LAB1
type MapF func(filename string, contents string) []KeyValue

// jobPhase indicates whether a task is scheduled as a map or reduce task.
type jobPhase string

const (
	mapPhase    jobPhase = "mapPhase"
	reducePhase          = "reducePhase"
)

type task struct {
	jobName      string
	file         MemFile            // only for map, the input file
	phase        jobPhase           // are we in mapPhase or reducePhase?
	taskNumber   int                // this task's index in the current phase
	nMap         int                // number of map tasks
	nReduce      int                // number of reduce tasks
	mapF         MapF               // map function used in this job
	reduceF      ReduceF            // reduce function used in this job
	reduceFiles  map[string]MemFile // used to store intermediate results produced by map task
	reduceResult MemFile            // used to store result produced by reduce task
	wg           sync.WaitGroup
}

// MRCluster represents a map-reduce cluster.
type MRCluster struct {
	nWorkers int
	wg       sync.WaitGroup
	taskCh   chan *task
	exit     chan struct{}
}

var singleton = &MRCluster{
	nWorkers: runtime.NumCPU(),
	taskCh:   make(chan *task),
	exit:     make(chan struct{}),
}

func init() {
	singleton.Start()
}

// GetMRCluster returns a reference to a MRCluster.
func GetMRCluster() *MRCluster {
	return singleton
}

// NWorkers returns how many workers there are in this cluster.
func (c *MRCluster) NWorkers() int { return c.nWorkers }

// Start starts this cluster.
func (c *MRCluster) Start() {
	for i := 0; i < c.nWorkers; i++ {
		c.wg.Add(1)
		go c.worker()
	}
}

func (c *MRCluster) worker() {
	defer c.wg.Done()
	for {
		select {
		case t := <-c.taskCh:
			if t.phase == mapPhase {
				files := make([]MemFile, t.nReduce)
				for i := range files {
					files[i] = CreateMemFile(reduceName(t.jobName, t.taskNumber, i))
				}
				results := t.mapF(t.file.Name(), t.file.Content())
				for _, kv := range results {
					enc := json.NewEncoder(files[ihash(kv.Key)%t.nReduce])
					if err := enc.Encode(&kv); err != nil {
						log.Fatalln(err)
					}
				}
				t.reduceFiles = make(map[string]MemFile, len(files))
				for _, f := range files {
					t.reduceFiles[f.Name()] = f
				}
			} else {
				data := make(map[string][]string, 64)
				for i := 0; i < t.nMap; i++ {
					reduceFile := reduceName(t.jobName, i, t.taskNumber)
					file, ok := t.reduceFiles[reduceFile]
					if !ok {
						log.Fatalln("reduce file not exist", reduceFile)
					}
					dec := json.NewDecoder(file)
					for {
						var kv KeyValue
						if err := dec.Decode(&kv); err != nil {
							if err == io.EOF {
								break
							} else {
								log.Fatalln("Decode reduce file error", err)
							}
						}
						data[kv.Key] = append(data[kv.Key], kv.Value)
					}
				}
				keys := make([]string, 0, len(data))
				for key := range data {
					keys = append(keys, key)
				}
				sort.Strings(keys)
				t.reduceResult = CreateMemFile(mergeName(t.jobName, t.taskNumber))
				buf := bufio.NewWriter(t.reduceResult)
				for _, k := range keys {
					v := t.reduceF(k, data[k])
					if _, err := buf.WriteString(v); err != nil {
						log.Fatalln(err)
					}
				}
				if err := buf.Flush(); err != nil {
					log.Fatalln(err)
				}
			}
			t.wg.Done()
		case <-c.exit:
			return
		}
	}
}

// Shutdown shutdowns this cluster.
func (c *MRCluster) Shutdown() {
	close(c.exit)
	c.wg.Wait()
}

// Submit submits a job to this cluster.
func (c *MRCluster) Submit(jobName string, mapF MapF, reduceF ReduceF, files []MemFile, nReduce int) <-chan []MemFile {
	notify := make(chan []MemFile)
	go c.run(jobName, mapF, reduceF, files, nReduce, notify)
	return notify
}

func (c *MRCluster) run(jobName string, mapF MapF, reduceF ReduceF, files []MemFile, nReduce int, notify chan<- []MemFile) {
	// map phase
	nMap := len(files)
	tasks := make([]*task, 0, nMap)
	for i := 0; i < nMap; i++ {
		t := &task{
			jobName:    jobName,
			file:       files[i],
			phase:      mapPhase,
			taskNumber: i,
			nReduce:    nReduce,
			nMap:       nMap,
			mapF:       mapF,
		}
		t.wg.Add(1)
		tasks = append(tasks, t)
		go func() { c.taskCh <- t }()
	}
	reduceFiles := make(map[string]MemFile, 32)
	for _, t := range tasks {
		t.wg.Wait()
		for _, f := range t.reduceFiles {
			reduceFiles[f.Name()] = f
		}
	}

	// reduce phase
	tasks = tasks[:0]
	for i := 0; i < nReduce; i++ {
		t := &task{
			jobName:     jobName,
			phase:       reducePhase,
			taskNumber:  i,
			nReduce:     nReduce,
			nMap:        nMap,
			reduceFiles: reduceFiles,
			reduceF:     reduceF,
		}
		t.wg.Add(1)
		tasks = append(tasks, t)
		go func() { c.taskCh <- t }()
	}
	reduceResults := make([]MemFile, 0, nReduce)
	for _, t := range tasks {
		t.wg.Wait()
		reduceResults = append(reduceResults, t.reduceResult)
	}
	notify <- reduceResults
}

func ihash(s string) int {
	h := fnv.New32a()
	h.Write([]byte(s))
	return int(h.Sum32() & 0x7fffffff)
}

// reduceName constructs the name of the intermediate file which map task
// <mapTask> produces for reduce task <reduceTask>.
func reduceName(jobName string, mapTask int, reduceTask int) string {
	return "mrtmp." + jobName + "-" + strconv.Itoa(mapTask) + "-" + strconv.Itoa(reduceTask)
}

// mergeName constructs the name of the output file of reduce task <reduceTask>
func mergeName(jobName string, reduceTask int) string {
	return "mrtmp." + jobName + "-res-" + strconv.Itoa(reduceTask)
}
