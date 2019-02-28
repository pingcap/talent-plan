export RUSTFLAGS=-Dwarnings
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=1

LOG_LEVEL ?= lab6824=info

check:
	cargo fmt --all
	cargo clippy --all --tests -- -D clippy::all

test_2a: cargo_test_2a

test_2b: cargo_test_2b

test_2c: cargo_test_2c

test_3a: cargo_test_3a

test_3b: cargo_test_3b

cargo_test_%:
	RUST_LOG=${LOG_LEVEL} cargo test --lib -- --nocapture --test $*

test_others:
	RUST_LOG=${LOG_LEVEL} cargo test -p labrpc -p labcodec -- --nocapture