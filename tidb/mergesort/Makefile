GO        := GO111MODULE=on go

.PHONY: all

all: test bench

test:
	$(GO) test -v

bench:
	$(GO) test -bench Benchmark -run xx -count 5 -check.b

