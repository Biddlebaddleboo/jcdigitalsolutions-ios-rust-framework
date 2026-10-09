#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --all -- --check
cargo test --locked -p framework-metal
cargo check --locked -p framework-metal --no-default-features
cargo clippy --locked -p framework-metal --all-targets --no-default-features -- -D warnings
cargo doc --locked -p framework-metal -p ios-metal --no-deps

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-metal --target "$target"
    cargo clippy --locked -p ios-metal --all-targets --target "$target" -- -D warnings
done

sh platform/ios/ios-metal/check-link-imports.sh
cargo xtask docs-check
git diff --check
