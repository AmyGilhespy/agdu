# justfile

profile := "debug"

[private]
default: list

# List all recipes.
list:
	just --list

# Build with `just build` -> dev, or `just --set profile release build` -> release
build:
	clear
	cargo build {{ if profile == "release" { "--release" } else { "" } }}

# Shorthand for building with godot crate feature
build-godot:
	clear
	cargo build --features=godot

# Shorthand for building in debug mode
build-debug:
	clear
	cargo build

# Shorthand for building in debug mode with godot crate feature
build-debug-godot:
	clear
	cargo build --features=godot

# Shorthand for building in release mode
build-release:
	clear
	cargo build --release

# Shorthand for building in release mode with godot crate feature
build-release-godot:
	clear
	cargo build --release --features=godot

# Clear cache, delete temps, and delete the built files.
clean:
	cargo clean

# Reformat the code as defined in the style guide.
fmt:
	find src -name '*.rs' -exec sh -c 'unexpand --first-only --tabs=4 {} > {}.fmt' ';' -exec mv '{}.fmt' '{}' ';'
	cargo fmt
	find src -name '*.rs' -exec sh -c 'unexpand --first-only --tabs=4 {} > {}.fmt' ';' -exec mv '{}.fmt' '{}' ';'
	unexpand --first-only --tabs=4 .justfile > .justfile.fmt ; mv .justfile.fmt .justfile

# Run `cargo check`, `cargo fmt --check`, `cargo clippy -- -D warnings` and same with `-W clippy::pedantic`, `cargo test`, and `cargo build`.
ci:
	clear
	cargo check
	cargo check --no-default-features
	cargo check --features=godot
	cargo check --features=backtrace
	cargo check --all-features
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo clippy -- -D warnings -W clippy::pedantic
	cargo clippy --no-default-features -- -D warnings
	cargo clippy --no-default-features -- -D warnings -W clippy::pedantic
	cargo clippy --features=godot -- -D warnings
	cargo clippy --features=godot -- -D warnings -W clippy::pedantic
	cargo clippy --all-features -- -D warnings
	cargo clippy --all-features -- -D warnings -W clippy::pedantic
	cargo test
	cargo test --no-default-features
	cargo test --features=godot
	cargo test --all-features
	cargo build
	cargo build --no-default-features
	cargo build --features=godot
	cargo build --all-features

publish-dry-run: ci
	cargo publish -p agdu --dry-run

publish-for-real: publish-dry-run
	cargo publish -p agdu
