#!/bin/sh
set -eu

cargo check --locked -p ios-bluetooth --target aarch64-apple-ios
cargo clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios -- -D warnings
cargo check --locked -p ios-bluetooth --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-bluetooth --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo test --locked --no-run -p ios-bluetooth --test connection_link --target aarch64-apple-ios
cargo test --locked --no-run -p ios-bluetooth --test connection_link --target aarch64-apple-ios-sim
