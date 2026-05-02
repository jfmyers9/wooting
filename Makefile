.PHONY: check fmt clippy test run-info run-test run-effect config-dry-run install-dry-run uninstall-dry-run

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
	cargo run -- test --brightness 96 --seconds 3

run-effect:
	cargo run -- effect comet --palette cyberpunk --brightness 128 --seconds 10 --fps 30

config-dry-run:
	cargo run -- run --config examples/wooting-hack.toml --dry-run

install-dry-run:
	scripts/install-macos.sh

uninstall-dry-run:
	scripts/uninstall-macos.sh
