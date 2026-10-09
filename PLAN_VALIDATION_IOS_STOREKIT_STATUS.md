# G54 — Legacy StoreKit purchase-status gate

## Scope

Gate D55/B60's iOS-only `SKPaymentQueue::canMakePayments` query for row 087. The wrapper is
deprecated with the Apple API and does not create a queue, show UI, inspect a product/account, or
process a transaction.

## Commands

- `sh platform/ios/ios-storekit-status/scripts/check.sh`
- `sh platform/ios/ios-storekit-status/scripts/check-link-imports.sh`

## Evidence boundary

The package gate covers host/device/Simulator compilation and strict Clippy, rustdoc, source and
feature guards, documentation and zero-Swift gates, and exact device/Simulator Release link-import
inspection. The link probes are built and inspected, never executed. These checks do not establish
live purchase ability, product availability, account state, or transaction success. No tests are
added or run by this gate.

## Status

The package gate passed in the isolated worktree and again in the integrated checkout on Rust 1.94.1,
Xcode 26.6 build 17F113, and iPhoneOS/iPhoneSimulator SDK 26.5. Host, device, and Simulator
compilation/strict Clippy, rustdoc, source/feature guards, exact Release import audits, docs check, and
zero-Swift-source check passed. Probes were built and inspected, not executed. No live purchase or
transaction behavior is claimed; the plan's Xcode 27.x baseline is not met.
