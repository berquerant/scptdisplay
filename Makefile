.PHONY: default ci lint format fmt clippy test build

default: ci

test:
	cargo test -- --nocapture

format:
	cargo fmt --all -- --check

fmt: format

clippy:
	cargo clippy -- -D warnings

lint: format clippy

build:
	cargo build

ci: lint test build
