#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
cd "$root"

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked -p ios-image-io --target "$target"
    cargo clippy --locked -p ios-image-io --all-targets --target "$target" -- -D warnings
done

sh platform/ios/ios-image-io/check-link-imports.sh
