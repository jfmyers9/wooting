.PHONY: check fmt clippy test run-info run-test run-effect run-command-pulse config-dry-run command-pulse-dry-run github-ci-dry-run install-dry-run uninstall-dry-run

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

run-command-pulse:
	cargo run -- signal run command-pulse -- make check

config-dry-run:
	cargo run -- run --config examples/wooting-signals.toml --dry-run

command-pulse-dry-run:
	cargo run -- run --config examples/command-pulse.toml --dry-run

github-ci-dry-run:
	cargo run -- run --config examples/github-ci.toml --dry-run

install-dry-run:
	scripts/install-macos.sh

uninstall-dry-run:
	scripts/uninstall-macos.sh
