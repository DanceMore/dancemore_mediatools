.PHONY: build test lint clean

build:
	cargo build

test:
	cargo test

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean
