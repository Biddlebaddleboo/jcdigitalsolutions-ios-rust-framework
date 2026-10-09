#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt -p ios-widgetkit-reload -- --check
cargo +1.94.1 check -p ios-widgetkit-reload
cargo +1.94.1 clippy --locked --lib -p ios-widgetkit-reload -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-widgetkit-reload
cargo +1.94.1 check --locked -p ios-widgetkit-reload --target aarch64-apple-ios
cargo +1.94.1 clippy --locked --lib -p ios-widgetkit-reload --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 check --locked -p ios-widgetkit-reload --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-widgetkit-reload --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-widgetkit-reload --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-widgetkit-reload/check.sh
sh -n platform/ios/ios-widgetkit-reload/check-swiftcall.sh
sh -n platform/ios/ios-widgetkit-reload/check-link-imports.sh
sh platform/ios/ios-widgetkit-reload/check-swiftcall.sh
sh platform/ios/ios-widgetkit-reload/check-link-imports.sh
git diff --check -- platform/ios/ios-widgetkit-reload PLAN_CAPABILITIES_WIDGETKIT.md docs/ios/widgetkit-reload.md Cargo.lock
