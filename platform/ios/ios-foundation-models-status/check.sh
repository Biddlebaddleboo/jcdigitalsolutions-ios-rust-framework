#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt --all -- --check
cargo +1.94.1 check -p ios-foundation-models-status
cargo +1.94.1 clippy --locked --lib -p ios-foundation-models-status -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-foundation-models-status
cargo +1.94.1 check --locked -p ios-foundation-models-status --target aarch64-apple-ios
cargo +1.94.1 clippy --locked --lib -p ios-foundation-models-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 check --locked -p ios-foundation-models-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-foundation-models-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-foundation-models-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-foundation-models-status/check.sh
sh -n platform/ios/ios-foundation-models-status/check-swiftcall.sh
sh -n platform/ios/ios-foundation-models-status/check-link-imports.sh
sh platform/ios/ios-foundation-models-status/check-swiftcall.sh
sh platform/ios/ios-foundation-models-status/check-link-imports.sh
git diff --check -- platform/ios/ios-foundation-models-status PLAN_CAPABILITIES_FOUNDATION_MODELS.md docs/ios/foundation-models-status.md Cargo.lock
