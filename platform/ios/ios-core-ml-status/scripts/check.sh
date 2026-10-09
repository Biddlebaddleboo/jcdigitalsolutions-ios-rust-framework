#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-core-ml-status/src/lib.rs
if ! rg -q 'unsafe \{ MLModel::availableComputeDevices\(\) \}' "$source_file"; then
    printf '%s\n' 'missing the Core ML device availability getter' >&2
    exit 1
fi
if ! rg -q 'objc2::available!\(ios = 17\.0, \.\.\)' "$source_file"; then
    printf '%s\n' 'missing the iOS 17.0 availability guard' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope Core ML call found' >&2
    exit 1
fi
if rg -n 'MLModel::(modelWith|load|prediction|predictions|new|init)|MLModelConfiguration::|MLFeatureProvider::' "$source_file"; then
    printf '%s\n' 'model load, inference, or input API found' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-core-ml-status/Cargo.toml -- --check
cargo check --locked -p ios-core-ml-status
cargo clippy --locked -p ios-core-ml-status --all-targets -- -D warnings
cargo doc --locked -p ios-core-ml-status --no-deps
cargo check --locked -p ios-core-ml-status --target aarch64-apple-ios
cargo check --locked -p ios-core-ml-status --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-core-ml-status --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-core-ml-status --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-core-ml-status --target aarch64-apple-ios --no-deps
sh platform/ios/ios-core-ml-status/scripts/check-link-imports.sh
cargo xtask zero-swift-source
git diff --check
