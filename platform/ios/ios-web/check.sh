#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$repo_root"
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-web --target "$target"
    cargo clippy --locked --all-targets -p ios-web --target "$target" -- -D warnings
done
