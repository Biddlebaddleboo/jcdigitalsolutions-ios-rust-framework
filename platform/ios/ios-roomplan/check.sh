#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo fmt --all -- --check
cargo check --locked --no-default-features -p framework-roomplan
cargo clippy --locked --no-default-features -p framework-roomplan -- -D warnings
cargo doc --locked --no-deps --no-default-features -p framework-roomplan
cargo check --locked -p ios-roomplan
cargo clippy --locked -p ios-roomplan -- -D warnings
cargo doc --locked --no-deps -p ios-roomplan
cargo check --locked --target aarch64-apple-ios -p framework-roomplan -p ios-roomplan
cargo clippy --locked --target aarch64-apple-ios -p framework-roomplan -p ios-roomplan -- -D warnings
cargo check --locked --target aarch64-apple-ios-sim -p framework-roomplan -p ios-roomplan
cargo clippy --locked --target aarch64-apple-ios-sim -p framework-roomplan -p ios-roomplan -- -D warnings
sh -n platform/ios/ios-roomplan/check.sh
sh -n platform/ios/ios-roomplan/check-swiftcall.sh
sh -n platform/ios/ios-roomplan/check-link-imports.sh
sh platform/ios/ios-roomplan/check-swiftcall.sh
sh platform/ios/ios-roomplan/check-link-imports.sh
git diff --check
