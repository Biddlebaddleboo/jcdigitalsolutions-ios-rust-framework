#!/bin/sh
set -eu

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

backend_source=platform/ios/ios-accessory/src/platform.rs
if ! rg -q 'EAAccessoryManager::sharedAccessoryManager\(\)' "$backend_source"; then
    printf '%s\n' 'expected the documented shared accessory manager query' >&2
    exit 1
fi
if ! rg -q 'manager\.connectedAccessories\(\)' "$backend_source"; then
    printf '%s\n' 'expected the connected-accessory snapshot getter' >&2
    exit 1
fi
if rg -n 'EASession|showBluetoothAccessoryPicker|registerForLocalNotifications|unregisterForLocalNotifications|EAAccessoryDelegate|protocolStrings|inputStream|outputStream|sendMessage|transfer[A-Za-z]*' "$backend_source"; then
    printf '%s\n' 'forbidden picker, event, session, protocol, or communication API found' >&2
    exit 1
fi

cargo fmt --manifest-path crates/framework-accessory/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-accessory/Cargo.toml -- --check
cargo check --locked -p framework-accessory --no-default-features
cargo clippy --locked -p framework-accessory --no-default-features -- -D warnings
cargo doc --locked -p framework-accessory --no-deps
cargo check --locked -p ios-accessory --target aarch64-apple-ios
cargo check --locked -p ios-accessory --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-accessory --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-accessory --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-accessory --target aarch64-apple-ios --no-deps
sh platform/ios/ios-accessory/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
