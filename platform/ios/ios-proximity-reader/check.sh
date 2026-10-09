#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo fmt --all -- --check
cargo check --locked -p ios-proximity-reader
cargo clippy --locked -p ios-proximity-reader -- -D warnings
cargo doc --locked --no-deps -p ios-proximity-reader
cargo check --locked --target aarch64-apple-ios -p ios-proximity-reader
cargo clippy --locked --target aarch64-apple-ios -p ios-proximity-reader -- -D warnings
cargo check --locked --target aarch64-apple-ios-sim -p ios-proximity-reader
cargo clippy --locked --target aarch64-apple-ios-sim -p ios-proximity-reader -- -D warnings
sh -n platform/ios/ios-proximity-reader/check.sh
sh -n platform/ios/ios-proximity-reader/check-swiftcall.sh
sh -n platform/ios/ios-proximity-reader/check-link-imports.sh
sh platform/ios/ios-proximity-reader/check-swiftcall.sh
sh platform/ios/ios-proximity-reader/check-link-imports.sh
git diff --check
