#!/bin/sh
set -eu

cargo fmt --manifest-path crates/framework-audio/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-playback/Cargo.toml -- --check
cargo check --locked -p ios-playback --target aarch64-apple-ios
cargo check --locked -p ios-playback --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-playback --lib --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-playback --lib --target aarch64-apple-ios-sim -- -D warnings
cargo check --locked -p ios-playback --target x86_64-apple-darwin
cargo clippy --locked -p ios-playback --lib --target x86_64-apple-darwin -- -D warnings
cargo doc --locked --no-deps -p framework-audio -p ios-playback
sh platform/ios/ios-playback/check-link-imports.sh
