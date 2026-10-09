#!/bin/sh
set -eu
package_dir=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$package_dir" rev-parse --show-toplevel)
cd "$repo_root"
package=ios-sign-in-with-apple-status

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo check --locked --offline -p "$package" --target "$target"
    cargo clippy --locked --offline -p "$package" --target "$target" -- -D warnings
done

RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline -p "$package" --target aarch64-apple-ios --no-deps
"$package_dir/check-link-imports.sh"
