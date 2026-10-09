#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --package framework-auth --package ios-auth -- --check
cargo test --locked -p framework-auth
cargo check --locked -p framework-auth --no-default-features
cargo clippy --locked -p framework-auth --all-targets -- -D warnings

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-auth --target "$target"
    cargo clippy --locked -p ios-auth --all-targets --target "$target" -- -D warnings
done

git diff --check
