# D58: StoreKit 2 purchase ability

## Scope

D58 adds one iOS-only status function for capability row 086, `ios_storekit2_status::can_make_payments() -> bool`. It reads `AppStore.canMakePayments` and adds no portable commerce contract

The slice complements D55/B60, which keeps the deprecated StoreKit 1 status query for compatibility. It adds no product, transaction, account, entitlement, or purchase flow

## Boundary

- Return the `AppStore.canMakePayments` Boolean
- Use only the public Swift property symbol `_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ`
- Gate the call with a C `swiftcall` thunk and weak import
- Return `false` when the weak symbol has no definition or on a non-iOS target
- The StoreKit 2 API and symbol are available from iOS 15.0. This is the API-availability floor,
  not the deployment minimum recorded by the Rust link probes

## Exclusions

- No product lookup, purchase, verification, restore, transaction update stream, or account query
- No StoreKit view, payment sheet, prompt, external purchase, or network operation
- No checked-in Swift source or claim of general Swift ABI support
- No change to D55/B60, PassKit row 088, or RoomPlan D56/B61

## Acceptance

- Confirm the Swift interface declaration and the exported StoreKit SDK symbol with a transient Swift oracle
- Confirm the Swift property call uses `swiftcc i1 ()` for device and Simulator
- Compare Clang C IR to the same `swiftcc i1 ()` call shape
- Build Rust-to-C-to-StoreKit link probes for iOS device and Simulator; do not execute them
- Require only StoreKit and `libSystem.B.dylib` direct imports, with StoreKit marked weak
- Pass strict Clippy, rustdoc, format, docs, zero-Swift, and diff checks
- Leave tests, live purchase checks, and runtime parity out of scope. Root integration records the
  backend and verified API metadata in row 086, changing the B/X row counts by one

## Evidence and limits

The local oracle used Xcode 26.6, Swift 6.3.3, and iOS SDK 26.5. This toolchain is below the Xcode 27.x plan baseline. The SDK marks the StoreKit 2 API and symbol available from iOS 15.0. Release link probes use the Rust target defaults: device minos 10.0 and Simulator minos 14.0. StoreKit.framework and the property symbol are weak imports; Clang IR and arm64 assembly show a null check before the call and a `false` result when the symbol is absent. This fallback is static compiler/link evidence only, not runtime-tested on an OS below iOS 15.0. No live payment query was made

- [AppStore.canMakePayments](https://developer.apple.com/documentation/storekit/appstore/canmakepayments)
- [AppStore](https://developer.apple.com/documentation/storekit/appstore)
