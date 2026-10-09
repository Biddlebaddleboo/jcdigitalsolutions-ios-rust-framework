#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt -p ios-dockkit-status -- --check
cargo +1.94.1 check -p ios-dockkit-status
cargo +1.94.1 clippy --locked --lib -p ios-dockkit-status -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-dockkit-status
cargo +1.94.1 check --locked -p ios-dockkit-status --target aarch64-apple-ios
cargo +1.94.1 clippy --locked --lib -p ios-dockkit-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 check --locked -p ios-dockkit-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-dockkit-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-dockkit-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-dockkit-status/check.sh
sh -n platform/ios/ios-dockkit-status/check-swiftcall.sh
sh -n platform/ios/ios-dockkit-status/check-link-imports.sh
sh platform/ios/ios-dockkit-status/check-swiftcall.sh
sh platform/ios/ios-dockkit-status/check-link-imports.sh
git diff --check -- platform/ios/ios-dockkit-status PLAN_CAPABILITIES_DOCKKIT.md docs/ios/dockkit-status.md Cargo.lock
