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
transaction behavior is claimed; that local run is below the planned Xcode 27.x baseline.

## Hosted Xcode 27 qualification

- [Run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431), source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; overall result `success`
- `Rust checks (xcode-27)` used Xcode `27.0` (`27A266a`), iPhoneOS `27.0` and iPhoneSimulator `27.0` SDKs; `Check legacy StoreKit purchase status` ran `sh platform/ios/ios-storekit-status/scripts/check.sh` and passed
- `Rust checks (macos-15)` also passed this package gate, but that runner reported Xcode `16.4`; `Rust checks (ubuntu-24.04)` skipped the Apple-only gate
- No diff from the run SHA to current main `cb807e43b2693f950293599f729d4445e8f827d6` in `platform/ios/ios-storekit-status`, `.github/workflows/ci.yml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, or `.cargo/config.toml`
- Hosted evidence covers static, compile, and link checks only; no live purchase ability, product/account state, queue creation, UI, or transaction result
