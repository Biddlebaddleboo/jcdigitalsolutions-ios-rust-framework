#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-storekit2-status/src/lib.rs
bridge_file=platform/ios/ios-storekit2-status/src/storekit2_bridge.c
if ! rg -q 'AppStore\.canMakePayments' "$source_file"; then
    echo 'no StoreKit 2 purchase-ability API docs found' >&2
    exit 1
fi
if ! rg -q 'swiftcall, weak_import' "$bridge_file" || ! rg -q '== 0' "$bridge_file"; then
    echo 'no weak StoreKit symbol guard found' >&2
    exit 1
fi
if rg -n 'SKPaymentQueue|SKPayment|Product::|Product\.purchase|Transaction|StoreView|purchase\(' "$source_file" "$bridge_file"; then
    echo 'out-of-scope StoreKit operation found' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-storekit2-status/Cargo.toml -- --check
cargo check --locked -p ios-storekit2-status
cargo clippy --locked --all-targets -p ios-storekit2-status -- -D warnings
cargo doc --locked -p ios-storekit2-status --no-deps
cargo check --locked -p ios-storekit2-status --target aarch64-apple-ios
cargo check --locked -p ios-storekit2-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-storekit2-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-storekit2-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-storekit2-status --target aarch64-apple-ios --no-deps
sh platform/ios/ios-storekit2-status/scripts/check-swift-abi.sh
sh platform/ios/ios-storekit2-status/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
