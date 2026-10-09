#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")/../.." rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --manifest-path platform/ios/ios-family-controls-status/Cargo.toml -- --check
cargo check --locked -p ios-family-controls-status
cargo clippy --locked --all-targets -p ios-family-controls-status -- -D warnings
cargo doc --locked -p ios-family-controls-status --no-deps
cargo check --locked -p ios-family-controls-status --target aarch64-apple-ios
cargo check --locked -p ios-family-controls-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-family-controls-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-family-controls-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-family-controls-status --target aarch64-apple-ios --no-deps
sh platform/ios/ios-family-controls-status/scripts/check-swift-abi.sh
sh platform/ios/ios-family-controls-status/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
