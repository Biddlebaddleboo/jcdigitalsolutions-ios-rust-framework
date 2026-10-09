#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt --all -- --check
cargo +1.94.1 check -p ios-matter-support-status
cargo +1.94.1 clippy --locked --lib -p ios-matter-support-status -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-matter-support-status
cargo +1.94.1 check --locked -p ios-matter-support-status --target aarch64-apple-ios
cargo +1.94.1 clippy --locked --lib -p ios-matter-support-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 check --locked -p ios-matter-support-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-matter-support-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-matter-support-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-matter-support-status/check.sh
sh -n platform/ios/ios-matter-support-status/check-swiftcall.sh
sh platform/ios/ios-matter-support-status/check-swiftcall.sh
git diff --check -- platform/ios/ios-matter-support-status PLAN_CAPABILITIES_MATTERSUPPORT.md docs/ios/matter-support-status.md Cargo.lock
