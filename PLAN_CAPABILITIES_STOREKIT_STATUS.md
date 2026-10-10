# D55: Legacy StoreKit status query

## Scope

D55 adds one iOS-only status API for capability row 087 in `platform/ios/ios-storekit-status`. It adds no portable contract and makes no change to PassKit row 088

The API is `ios_storekit_status::legacy_can_make_payments() -> bool`. It calls only `SKPaymentQueue::canMakePayments`

Apple marks `SKPaymentQueue` unsupported and marks `canMakePayments` as deprecated since iOS 18.0; StoreKit 2 `AppStore.canMakePayments` is the current alternative. This slice keeps only a legacy status query for compatibility; it does not add StoreKit 1 transaction support

## Acceptance

- `ios-storekit-status` exports only `legacy_can_make_payments` on iOS
- The function returns `false` below the iOS 3.0 API floor
- Rust docs and the guide state the StoreKit 1 deprecation and the StoreKit 2 replacement
- The backend calls only `SKPaymentQueue::canMakePayments`; it creates no queue or payment UI, and it reads no product, account, transaction, or entitlement data
- The package uses `objc2-store-kit` 0.3.2 with default features off and only `SKPaymentQueue`
- The direct import set is `Foundation`, `StoreKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`
- The package, device, Simulator, strict Clippy, rustdoc, import, source-guard, docs, zero-Swift, and diff gates pass
- No tests run; link probe executables are built and inspected only

## Validation

Run `sh platform/ios/ios-storekit-status/scripts/check.sh`

## D55 completion evidence — 2026-10-10

Reviewed and completed from base `1ac40c6159cdfc480d9c05eabc522834b6df2066`. The product implementation was already present and met the scope; no API, package, guide, matrix, or manifest change was needed. `src/lib.rs` exports only `legacy_can_make_payments` on iOS, returns `false` below iOS 3.0, and calls only `SKPaymentQueue::canMakePayments`. Its Rust docs and `docs/ios/storekit-status.md` state StoreKit 1 deprecation and the StoreKit 2 replacement and exclude queue, product, account, transaction, entitlement, and payment UI behavior. The package selects `objc2-store-kit` 0.3.2 with workspace default features disabled and only `SKPaymentQueue` enabled.

Installed and inspected pinned `ios-rust-build` and `ios-rust-validate` 0.1.0 (`source_sha=2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`, Rust 1.94.1). `ios-rust-validate --list` does not register this package, so its package-owned `scripts/check.sh` is the applicable gate. `sh platform/ios/ios-storekit-status/scripts/check.sh` passed: format, host/device/Simulator package checks, strict Clippy, host/device rustdoc, Release device/Simulator link-probe builds and import/source inspection (exactly Foundation, StoreKit, libSystem.B.dylib, libobjc.A.dylib; no Swift runtime), docs, zero-Swift, and diff checks.

No tests were run. Device and Simulator probe binaries were built and inspected only; they were not executed. This records compile/link/import evidence, not runtime StoreKit behavior or live payment eligibility.
