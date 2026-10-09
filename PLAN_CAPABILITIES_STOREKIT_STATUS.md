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
