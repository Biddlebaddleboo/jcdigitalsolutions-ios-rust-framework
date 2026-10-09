#!/bin/sh
set -eu

repo_root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$repo_root"

source_file=crates/framework-payments/src/ios.rs
if ! rg -q 'unsafe \{ PKPaymentAuthorizationController::canMakePayments\(\) \}' "$source_file"; then
    printf '%s\n' 'missing the canMakePayments status query' >&2
    exit 1
fi
unsafe_calls=$(rg -c 'unsafe \{' "$source_file")
if [ "$unsafe_calls" -ne 1 ]; then
    printf '%s\n' 'out-of-scope PassKit call found' >&2
    exit 1
fi
if rg -n '::(alloc|new|canMakePaymentsUsingNetworks|canMakePaymentsUsingNetworks_capabilities|presentWithCompletion|initWithPaymentRequest|setDelegate|delegate)|PKPaymentRequest|PKPaymentNetwork|PKMerchantCapability|PKPaymentButton|PKPaymentAuthorizationViewController' "$source_file"; then
    printf '%s\n' 'payment UI, card, merchant, or transaction API found' >&2
    exit 1
fi

cargo fmt --manifest-path crates/framework-payments/Cargo.toml -- --check
cargo check --locked -p framework-payments
cargo clippy --locked -p framework-payments -- -D warnings
cargo doc --locked -p framework-payments --no-deps
cargo check --locked -p framework-payments --target aarch64-apple-ios
cargo check --locked -p framework-payments --target aarch64-apple-ios-sim
cargo clippy --locked -p framework-payments --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p framework-payments --target aarch64-apple-ios-sim -- -D warnings
cargo doc --locked -p framework-payments --target aarch64-apple-ios --no-deps
sh crates/framework-payments/scripts/check-link-imports.sh
cargo xtask docs-check
cargo xtask zero-swift-source
git diff --check
