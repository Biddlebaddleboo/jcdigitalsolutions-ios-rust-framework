#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"

cargo +1.94.1 fmt -p ios-accessory-setup-status -- --check
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 check --locked -p ios-accessory-setup-status
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 clippy --locked --lib -p ios-accessory-setup-status -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-accessory-setup-status
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 check --locked -p ios-accessory-setup-status --target aarch64-apple-ios
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 clippy --locked --lib -p ios-accessory-setup-status --target aarch64-apple-ios -- -D warnings
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 check --locked -p ios-accessory-setup-status --target aarch64-apple-ios-sim
IPHONEOS_DEPLOYMENT_TARGET=18.0 cargo +1.94.1 clippy --locked --lib -p ios-accessory-setup-status --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked --no-deps -p ios-accessory-setup-status --target aarch64-apple-ios
cargo +1.94.1 xtask docs-check
sh -n platform/ios/ios-accessory-setup-status/check.sh
git diff --check -- platform/ios/ios-accessory-setup-status PLAN_CAPABILITIES_ACCESSORY_SETUP.md docs/ios/accessory-setup-status.md Cargo.lock
