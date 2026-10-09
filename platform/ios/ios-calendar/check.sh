#!/bin/sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
cd "$repo_root"

cargo fmt --all -- --check
cargo test -p framework-calendar --locked
cargo check -p framework-calendar --no-default-features --locked
cargo test -p ios-calendar --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p framework-calendar -p ios-calendar --no-deps --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p ios-calendar --target aarch64-apple-ios --no-deps --locked
cargo check -p ios-calendar --target aarch64-apple-ios --locked
cargo check -p ios-calendar --target aarch64-apple-ios-sim --locked
cargo clippy -p ios-calendar --all-targets --target aarch64-apple-ios --locked -- -D warnings
cargo clippy -p ios-calendar --all-targets --target aarch64-apple-ios-sim --locked -- -D warnings
platform/ios/ios-calendar/check-link-imports.sh
cargo run --locked --quiet -p xtask -- docs-check
git diff --check
