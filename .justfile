# just manual: https://github.com/casey/just/#readme

package := "enet-mqtt"
release-args := ("--release --package " + package + " --locked --bin " + package)
static-args := (release-args + " --features vendored")

_default:
    @just --list

# Runs clippy on the sources
lint:
	cargo clippy --locked -- -D warnings

# Runs unit tests
test:
	cargo test --locked
