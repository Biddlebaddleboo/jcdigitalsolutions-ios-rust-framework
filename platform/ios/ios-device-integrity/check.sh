#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --all -- --check
cargo check --locked -p framework-device-integrity --no-default-features
cargo clippy --locked -p framework-device-integrity --all-targets --no-default-features -- -D warnings
cargo doc --locked -p framework-device-integrity --no-deps

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-device-integrity --target "$target"
    cargo clippy --locked -p ios-device-integrity --all-targets --target "$target" -- -D warnings
done

sh platform/ios/ios-device-integrity/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
