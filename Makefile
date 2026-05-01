.PHONY: check fmt clippy test run-info run-test

check: fmt clippy test

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

run-info:
	cargo run -- info

run-test:
	cargo run -- test --brightness 16 --seconds 3
