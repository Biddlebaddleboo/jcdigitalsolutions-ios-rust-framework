#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --all -- --check
cargo test --locked -p framework-nearby
cargo check --locked -p framework-nearby --no-default-features
cargo clippy --locked -p framework-nearby --all-targets -- -D warnings

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-nearby --target "$target"
    cargo clippy --locked -p ios-nearby --all-targets --target "$target" -- -D warnings
done

git diff --check
