#!/bin/sh
set -eu

cargo fmt --manifest-path platform/ios/ios-media-authorization/Cargo.toml -- --check
cargo test --locked -p ios-media-authorization
cargo check --locked -p ios-media-authorization --target aarch64-apple-ios
cargo check --locked -p ios-media-authorization --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-media-authorization -- -D warnings
cargo clippy --locked --all-targets -p ios-media-authorization --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-media-authorization --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked --no-deps -p ios-media-authorization --target aarch64-apple-ios
sh platform/ios/ios-media-authorization/check-surface.sh
