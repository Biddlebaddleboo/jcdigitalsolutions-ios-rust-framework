# PLAN_BINDINGS_F32.md — F32: StoreKit 2 purchase-ability C ABI

## Objective

Expose only D58/B63's synchronous `AppStore.canMakePayments` status through one opt-in C export. Add no
portable commerce contract, StoreKit object, product, transaction, purchase, restore, payment UI, or network API

## Candidate and bounds

B63 already exposes `ios_storekit2_status::can_make_payments() -> bool`. Its Swift ABI call uses the
compiler-checked C `swiftcall` thunk at `platform/ios/ios-storekit2-status/src/storekit2_bridge.c`. The
StoreKit framework and `_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ` symbol are weak imports. The thunk checks
the symbol before a call and returns false if absent. This is a stable scalar call with no C-visible object,
handle, callback, request, or lifecycle

F32 does not broaden the query: true means StoreKit reports that the person can authorize purchases; false
may mean purchase restrictions or the absent weak symbol. It does not show product availability, account
identity, entitlement state, transaction success, or payment readiness

Other remaining candidates are not selected: B53 SafetyKit has an unresolved getter-specific entitlement
prerequisite; B55 GameKit returns signed local-player state; B60 is deprecated StoreKit 1 purchase ability.
They need a separate contract review. D58/B63 is the only existing candidate here with one unambiguous Boolean
and a compiler-checked weak-symbol fallback

## Exact C contract

- Cargo feature: `ios-storekit2-status`; default remains empty
- Header: `bindings/c/include/framework_ios_storekit2_status.h`
- Export: `FrameworkStatus framework_ios_storekit2_status_can_make_payments(uint8_t *out_can_make_payments)`
- On iOS, return `FRAMEWORK_STATUS_OK` and write exactly `0` or `1` from B63
- `false` can reflect purchase restrictions or the absent weak symbol; the absence fallback is static
  compiler/link evidence only and has no runtime check below iOS 15.0
- API and symbol floor: iOS 15.0. B63 device and Simulator link minima are 10.0 and 14.0; these are link
  settings, not the API floor. The weak import and null check support the lower probe minima without a claim
  of live behavior on older OS releases
- A valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` and leaves output zero
- Null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A caught panic returns
  `FRAMEWORK_STATUS_PANIC` and leaves output zero
- Output is one caller-owned byte, initialized to zero before platform handling. The API checks nullness
  only. A non-null pointer must be valid, aligned, writable for the full synchronous call; the caller must
  prevent unsynchronized access. The pointer is not retained
- The call runs on the caller's thread. No thread-affinity or thread-safety guarantee is added
- No StoreKit object, Swift object, product, transaction, callback, or native pointer crosses C

## Source evidence

- D58/B63 API, symbol, weak-import, minos, and toolchain evidence: `PLAN_CAPABILITIES_STOREKIT2_STATUS.md`,
  `PLAN_IOS_STOREKIT2_STATUS.md`, and `platform/ios/ios-storekit2-status`
- Public Apple API: [AppStore.canMakePayments](https://developer.apple.com/documentation/storekit/appstore/canmakepayments)
- B63 compiler-oracle and static link evidence: Swift 6.3.3 / Xcode 26.6 / iOS SDK 26.5; this toolchain is
  below the repository's Xcode 27.x baseline

## Files and root wiring

F32 owns `bindings/c/src/ios_storekit2_status.rs`,
`bindings/c/include/framework_ios_storekit2_status.h`,
`bindings/c/check-ios-storekit2-status.sh`,
`bindings/c/check-ios-storekit2-status-link.sh`, `docs/bindings/ios-storekit2-status.md`, and this plan.
The isolated implementation also adds the optional dependency/feature, module/re-export, and ABI manifest
entry to `bindings/c`. Root integration is complete: Cargo.lock, both macOS CI gates, aggregate plan
evidence, and the docs index are wired. The global capability matrix and counts did not change

## Validation and limits

F32's isolated validation passed with Rust 1.94.1, Xcode 26.6 build 17F113, and iPhoneOS/iPhoneSimulator
SDK 26.5. This toolchain is below the repository's Xcode 27.x baseline

- `cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features
  ios-storekit2-status` passed and refreshed only the isolated snapshot's Cargo.lock
- `sh bindings/c/check-ios-storekit2-status.sh` passed format, shell syntax, JSON and source/header contract,
  host/default/iOS feature-tree isolation, and standalone C11/C++17 header syntax
- `sh bindings/c/check-ios-storekit2-status-link.sh` passed locked host/device/Simulator checks, strict
  Clippy, host rustdoc, Release archive builds, C11/C++17 host/device/Simulator links, symbol parity, exact
  imports, weak import checks, and minos inspection
- Host C/C++ imported only `libSystem.B.dylib`. Device and Simulator C/C++ imported `StoreKit` as a weak
  framework plus `libSystem.B.dylib`; the StoreKit getter symbol is a weak external import. No direct Swift
  runtime import was present. Device minos was 10.0 and Simulator minos was 14.0
- Linked consumers were inspected but not executed. No tests, live StoreKit query, transaction, purchase, or
  runtime check below iOS 15.0 ran. No passing CI workflow run had been recorded at this validation point.
  Mainline run
  [38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
  later passed the F32 C ABI gates as part of steps 235–275 on macOS 15 and Xcode 27; linked probes
  were not executed, and no runtime or device evidence is claimed.
- Both gates passed in the integrated checkout after root wiring; linked consumers were inspected but not
  executed

F32 does not establish payment readiness, successful payment, account state, product or network state,
StoreKit 1 behavior, parity, or performance. The weak-symbol false fallback remains untested at runtime below
iOS 15.0
