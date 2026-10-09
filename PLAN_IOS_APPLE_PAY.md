# B51: PassKit Apple Pay capability status

## Status

`sh crates/framework-payments/scripts/check.sh` passed in the integrated checkout on Rust 1.94.1 /
Xcode 26.6 build 17F113 / iOS SDK 26.5. Device, Simulator, host, strict Clippy, rustdoc, source-scope,
and exact import gates passed. The linked probes were not executed; no live Apple Pay query or
transaction behavior is claimed. The package gate is wired in macOS CI; no passing CI workflow run
is recorded, and the host remains below the Xcode 27.x plan baseline

## Native boundary

Use only `PKPaymentAuthorizationController::canMakePayments` from `PassKit.framework`. The class is
available from iOS 10.0. The backend checks that runtime floor with `objc2::available!` and returns
`false` below it

The typed dependency is `objc2-pass-kit` 0.3.2 with default features off and only the
`PKPaymentAuthorizationController` feature. The expected direct imports are PassKit, Foundation,
`libobjc.A.dylib`, and `libSystem.B.dylib`; no UIKit or StoreKit import is allowed

The class query has no main-thread marker and no arguments. The Rust API exposes only a Boolean. A
true result means the device is generally capable of payments under its hardware and parental
control state; it does not establish card setup, merchant support, app access, payment approval, or
transaction completion

## Exclusions

Do not create a payment controller, request, sheet, or button. Do not inspect payment networks,
cards, merchant capabilities, passes, or accounts. Do not invoke StoreKit, authorize a transaction,
or handle payment data

## Validation

Run `sh crates/framework-payments/scripts/check.sh`. It performs host, iOS device, and iOS
Simulator checks, strict Clippy, rustdoc, source guards, and exact device/Simulator import checks.
It does not add or run tests or execute either link probe
