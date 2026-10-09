# B63: StoreKit 2 purchase ability status

## Purpose

B63 adds a Rust status function for D58 and row 086 through one Swift ABI call from a Clang C `swiftcall` thunk. It does not add StoreKit 2 purchase behavior

## API

- Crate: `ios-storekit2-status`
- Rust API: `ios_storekit2_status::can_make_payments() -> bool`
- Swift API: `AppStore.canMakePayments`
- StoreKit 2 API/symbol availability floor: iOS 15.0
- Framework: `StoreKit.framework`
- Swift symbol: `_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ`
- LLVM IR signature: `swiftcc i1 ()`
- C thunk: `framework_storekit2_can_make_payments() -> bool`

The C thunk marks the Swift symbol `swiftcall, weak_import`, checks for a definition, then calls it. If the weak symbol is absent, the thunk returns `false` without a call. This supports a lower link-probe minos without claiming that the StoreKit 2 API exists there. The API/symbol is available from iOS 15.0; the Release probes use the Rust target defaults of device minos 10.0 and Simulator minos 14.0. StoreKit.framework and the property symbol are weak imports. The null-check fallback is confirmed by compiler and link inspection only; no runtime check below iOS 15 took place

## Semantics and limits

- `true` means StoreKit reports that the person can authorize purchases
- `false` may reflect purchase restrictions or an absent weak symbol
- The value does not report products, account identity, entitlement state, or transaction success
- No StoreKit 1 queue, transaction, product, purchase, restore, UI, or network API is called
- The API is iOS-only; the host function returns `false`
- No Swift source, Swift object, or C++ code ships in this package
- The Swift symbol and ABI were confirmed only with Xcode 26.6 / Swift 6.3.3 / SDK 26.5, below the Xcode 27.x plan baseline

## Build and link evidence

The Swift source used for the API oracle is made at a temporary path outside the checkout, compiled to LLVM IR, and removed at script exit. Device and Simulator IR both call the exported StoreKit symbol as `swiftcc i1 ()`. The SDK interface marks `AppStore` available from iOS 15.0

Rust calls a C ABI thunk. The C thunk uses the public StoreKit symbol with `swiftcall`; no Swift source or Swift object is part of the package. Device and Simulator link probes direct-import only weak `StoreKit.framework` and `libSystem.B.dylib`; neither app binary has a direct Swift runtime import. Their linked minos are 10.0 (device) and 14.0 (Simulator), with SDK 26.5. The release binaries were inspected but never executed. The absent-symbol fallback has no pre-iOS-15 runtime evidence

## Out of scope

- StoreKit product, transaction, purchase, verification, restore, subscription, and update APIs
- StoreKit view or payment UI
- PassKit row 088 or the D55/B60 StoreKit 1 query
- Runtime payment, account, or purchase checks
