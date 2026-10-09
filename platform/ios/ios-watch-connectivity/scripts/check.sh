#!/bin/sh
set -eu

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

backend_source=platform/ios/ios-watch-connectivity/src/platform.rs
if ! rg -q 'WCSession::isSupported\(\)' "$backend_source"; then
    printf '%s\n' 'expected the sole native query WCSession::isSupported()' >&2
    exit 1
fi
if rg -n 'defaultSession|activateSession|isPaired|isWatchAppInstalled|isReachable|WCSessionDelegate|sendMessage|transfer[A-Za-z]*|updateApplicationContext' "$backend_source"; then
    printf '%s\n' 'forbidden session state or communication API found in the status-only adapter' >&2
    exit 1
fi
if rg -n 'WCSession::' "$backend_source" | rg -v 'WCSession::isSupported\(\)'; then
    printf '%s\n' 'unexpected WatchConnectivity binding call found in the status-only adapter' >&2
    exit 1
fi

cargo fmt --manifest-path crates/framework-watch-connectivity/Cargo.toml -- --check
cargo fmt --manifest-path platform/ios/ios-watch-connectivity/Cargo.toml -- --check
cargo check --locked -p framework-watch-connectivity --no-default-features
cargo clippy --locked -p framework-watch-connectivity --no-default-features -- -D warnings
cargo doc --locked -p framework-watch-connectivity --no-deps
cargo check --locked -p ios-watch-connectivity --target aarch64-apple-ios
cargo check --locked -p ios-watch-connectivity --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-watch-connectivity --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-watch-connectivity --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-watch-connectivity --target aarch64-apple-ios --no-deps
sh platform/ios/ios-watch-connectivity/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
