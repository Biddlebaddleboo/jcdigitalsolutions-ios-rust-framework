#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt --all -- --check
cargo +1.94.1 check --locked -p ios-photogrammetry-status
cargo +1.94.1 clippy --locked --lib -p ios-photogrammetry-status -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-photogrammetry-status
cargo +1.94.1 check --locked -p ios-photogrammetry-status --target aarch64-apple-ios
cargo +1.94.1 clippy --locked --lib -p ios-photogrammetry-status --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 check --locked -p ios-photogrammetry-status --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked --lib -p ios-photogrammetry-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-photogrammetry-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-photogrammetry-status/check.sh
sh -n platform/ios/ios-photogrammetry-status/check-swiftcall.sh
sh -n platform/ios/ios-photogrammetry-status/check-link-imports.sh
sh platform/ios/ios-photogrammetry-status/check-swiftcall.sh
sh platform/ios/ios-photogrammetry-status/check-link-imports.sh
git diff --check -- platform/ios/ios-photogrammetry-status PLAN_CAPABILITIES_REALITYKIT.md docs/ios/photogrammetry-status.md
