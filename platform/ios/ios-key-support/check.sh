#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

cargo check --locked --no-default-features -p framework-key-support
cargo clippy --locked --no-default-features -p framework-key-support -- -D warnings
cargo doc --locked --no-deps -p framework-key-support
cargo doc --locked --no-deps -p ios-key-support --target aarch64-apple-ios

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-key-support --target "$target"
    cargo clippy --locked --all-targets -p ios-key-support --target "$target" -- -D warnings
done

cargo fmt --package framework-key-support --package ios-key-support -- --check
sh -n platform/ios/ios-key-support/check.sh
sh -n platform/ios/ios-key-support/check-link-imports.sh
