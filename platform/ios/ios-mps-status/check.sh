#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

cargo fmt --manifest-path platform/ios/ios-mps-status/Cargo.toml --package ios-mps-status -- --check
cargo check --locked --all-targets -p ios-mps-status
cargo check --locked --all-targets -p ios-mps-status --target aarch64-apple-ios
cargo check --locked --all-targets -p ios-mps-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-mps-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-mps-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked --no-deps -p ios-mps-status --target aarch64-apple-ios

host_dependencies=$(cargo tree --locked -p ios-mps-status --target aarch64-apple-darwin)
if printf '%s\n' "$host_dependencies" | grep -F 'objc2-metal-performance-shaders'; then
    echo "Metal Performance Shaders binding must remain off non-iOS targets" >&2
    exit 1
fi

features=$(cargo tree --locked -p ios-mps-status --target aarch64-apple-ios -e features)
printf '%s\n' "$features" | grep -F 'objc2-metal-performance-shaders feature "MPSCore"' >/dev/null
if printf '%s\n' "$features" | grep -F 'objc2-metal-performance-shaders feature "default"'; then
    echo "objc2-metal-performance-shaders default features must remain disabled" >&2
    exit 1
fi

sh -n platform/ios/ios-mps-status/check-link-imports.sh
cargo xtask docs-check
git diff --check
