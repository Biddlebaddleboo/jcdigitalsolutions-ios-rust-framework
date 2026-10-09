# Legacy StoreKit status

`ios-storekit-status` exposes one iOS-only query

```rust
#[allow(deprecated)]
let can_make_payments = ios_storekit_status::legacy_can_make_payments();
```

The call reads only `SKPaymentQueue::canMakePayments`. It has an iOS 3.0 API floor and returns `false` below that floor. Apple deprecated this method at iOS 18.0 and lists StoreKit 2 `AppStore.canMakePayments` as the current alternative. The Rust API is also deprecated to mark that limit

The result is a point-in-time system purchase-ability bit. It does not prove that a product exists, identify an account or payment method, promise a successful payment, or report a transaction or entitlement. This call does not request a purchase or show payment UI

This package does not add a portable commerce contract or StoreKit 1 transaction support. See [D55](../../PLAN_CAPABILITIES_STOREKIT_STATUS.md) and [B60](../../PLAN_IOS_STOREKIT_STATUS.md)

The device and Simulator link gate allows only `Foundation`, `StoreKit`, `libSystem.B.dylib`, and `libobjc.A.dylib`. It rejects Swift runtime imports. The gate builds and inspects probes but does not run them or a payment flow

Apple sources: [SKPaymentQueue.canMakePayments](https://developer.apple.com/documentation/storekit/skpaymentqueue/canmakepayments%28%29), [SKPaymentQueue](https://developer.apple.com/documentation/storekit/skpaymentqueue?preferredLanguage=occ), and [AppStore.canMakePayments](https://developer.apple.com/documentation/storekit/appstore/canmakepayments)
