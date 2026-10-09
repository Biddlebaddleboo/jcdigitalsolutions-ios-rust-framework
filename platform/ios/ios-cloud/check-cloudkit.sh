#!/bin/sh
set -eu

cargo fmt --all -- --check
cargo check --locked -p framework-cloud --no-default-features
cargo clippy --locked -p framework-cloud --all-targets --no-default-features -- -D warnings
cargo check --locked -p ios-cloud --target aarch64-apple-ios
cargo check --locked -p ios-cloud --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-cloud --all-targets --target aarch64-apple-ios-sim -- -D warnings
sh platform/ios/ios-cloud/check-link-imports.sh device
sh platform/ios/ios-cloud/check-link-imports.sh simulator
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
