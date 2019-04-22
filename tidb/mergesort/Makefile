.PHONY: all

all: test bench

test:
	go test

bench:
	go test -bench Benchmark -run xx -count 5 -benchmem

mytest:
	go test -bench=. -cpuprofile=cpuprofile -run=xxx