#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=platform/ios/ios-storekit-status/src/lib.rs
if ! rg -q 'unsafe \{ SKPaymentQueue::canMakePayments\(\) \}' "$source_file"; then
    printf '%s\n' 'missing the legacy StoreKit canMakePayments query' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope StoreKit call found' >&2
    exit 1
fi
if rg -n '::(defaultQueue|addPayment|addTransactionObserver|removeTransactionObserver|restoreCompletedTransactions|finishTransaction|transactions|storefront|presentCodeRedemptionSheet|showPriceConsentIfNeeded)|\bSKPayment\b|\bSKProduct\b|\bSKPaymentTransaction\b|\bSKPaymentTransactionObserver\b|Product::|AppStore::' "$source_file"; then
    printf '%s\n' 'purchase, transaction, account, or UI API found' >&2
    exit 1
fi
if ! rg -q 'objc2-store-kit = \{ workspace = true, features = \["SKPaymentQueue"\] \}' platform/ios/ios-storekit-status/Cargo.toml; then
    printf '%s\n' 'unexpected objc2-store-kit feature set' >&2
    exit 1
fi

cargo fmt --manifest-path platform/ios/ios-storekit-status/Cargo.toml -- --check
cargo check --locked -p ios-storekit-status
cargo clippy --locked --all-targets -p ios-storekit-status -- -D warnings
cargo doc --locked -p ios-storekit-status --no-deps
cargo check --locked -p ios-storekit-status --target aarch64-apple-ios
cargo check --locked -p ios-storekit-status --target aarch64-apple-ios-sim
cargo clippy --locked --all-targets -p ios-storekit-status --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --all-targets -p ios-storekit-status --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p ios-storekit-status --target aarch64-apple-ios --no-deps
sh platform/ios/ios-storekit-status/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
