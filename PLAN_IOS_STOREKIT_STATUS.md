# B60: Legacy StoreKit purchase status

## Native API

- Call only `SKPaymentQueue::canMakePayments` from `StoreKit.framework`
- The API floor is iOS 3.0
- Apple marks `SKPaymentQueue` unsupported and this method as deprecated since iOS 18.0; Apple lists StoreKit 2 `AppStore.canMakePayments` as the current alternative
- Apple docs say this Boolean matches `AppStore.canMakePayments`; it reports general purchase ability only, not product availability or a successful transaction
- The method is a class method with no args; the backend does not create `SKPaymentQueue`, call `defaultQueue`, register a transaction event hook, or show payment UI
- Apple headers do not list a main-thread rule for this method; this crate makes no new thread claim
- The isolated workstream pinned `objc2-store-kit` 0.3.2 directly; root integration uses the
  central workspace pin with default features off and only `SKPaymentQueue`
- Expected direct imports: `Foundation`, `StoreKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`; no Swift runtime or unrelated framework import

## Exclusions

Do not add a purchase API, product query, transaction event hook, receipt read, restore flow, account status, entitlement status, payment sheet, or portable commerce type

Do not claim this legacy StoreKit 1 API is Apple's supported current purchase flow

## Validation

Run `sh platform/ios/ios-storekit-status/scripts/check.sh`. It runs host, iOS device, and iOS Simulator check, strict Clippy, rustdoc, source guards, and exact device/Simulator import audit. It builds but does not run either link probe and adds or runs no tests

## Apple sources

- [SKPaymentQueue.canMakePayments](https://developer.apple.com/documentation/storekit/skpaymentqueue/canmakepayments%28%29)
- [SKPaymentQueue](https://developer.apple.com/documentation/storekit/skpaymentqueue?preferredLanguage=occ)
- [AppStore.canMakePayments](https://developer.apple.com/documentation/storekit/appstore/canmakepayments)
