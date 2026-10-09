#!/bin/sh
set -eu

cargo fmt --manifest-path crates/framework-media-authorization/Cargo.toml -- --check
cargo check --locked --no-default-features -p framework-media-authorization
cargo test --locked -p framework-media-authorization
cargo clippy --locked --all-targets -p framework-media-authorization -- -D warnings
cargo doc --locked --no-deps -p framework-media-authorization
