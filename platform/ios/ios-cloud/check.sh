#!/bin/sh
set -eu

cargo fmt --all -- --check
cargo test --locked -p framework-cloud
cargo check --locked -p framework-cloud --no-default-features
cargo check --locked -p ios-cloud --target aarch64-apple-ios
cargo check --locked -p ios-cloud --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo xtask docs-check
git diff --check
