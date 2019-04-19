GO        := GO111MODULE=on go

.PHONY: all

all: test bench

test:
	$(GO) test -v

bench:
	$(GO) test -bench Benchmark -count 2 -v -check.b

