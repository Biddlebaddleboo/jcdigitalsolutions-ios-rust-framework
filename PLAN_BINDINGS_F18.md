# PLAN_BINDINGS_F18.md — F18: ProximityReader device-model C ABI

## Objective

Expose B76's existing non-prompting `PaymentCardReader.isSupported` query through one opt-in C function. Preserve its narrow device-model meaning and iOS 15.4 floor. Do not add a reader, session, NFC, UI, or payment API

## Status

F18 is integrated. After root refreshed Cargo.lock and corrected the focused gate's ripgrep invocation (plain `rg` treats patterns as regular expressions; `rg -E` is an encoding flag), `sh bindings/c/check-ios-proximity-reader.sh` passed host, device, and Simulator feature isolation, checks, strict Clippy, Release builds, C11/C++17 links, imports, Swift-symbol checks, and minos 15.4. Its static gate also asserts aligned writable output storage for the full call, caller protection from unsynchronized access, and no pointer retention. Link probes and consumers were not executed; no tests ran

## Write scope

- `bindings/c/src/ios_proximity_reader.rs`
- `bindings/c/include/framework_ios_proximity_reader.h`
- `bindings/c/check-ios-proximity-reader.sh`
- `docs/bindings/ios-proximity-reader.md`
- this plan

Root owns `bindings/c/Cargo.toml`, `bindings/c/src/lib.rs`, `bindings/c/abi-manifest.json`, Cargo.lock, CI, aggregate binding status, and documentation indexes. Do not edit `platform/ios/ios-proximity-reader` or any root integration surface

## Backend audit and feasibility

B76 exposes only `ios_proximity_reader::tap_to_pay_device_model_supported() -> Option<bool>`. On iOS it calls the public `PaymentCardReader.isSupported` Swift property through a compiler-checked C `swiftcall` thunk; non-iOS returns `None`. The property is iOS 15.4+, and the package's C shim compiles only for 64-bit Apple iOS device and Simulator. B76's gate verifies the generated Swift metadata response and `swift_context` getter lowering against temporary compiler-oracle output, then checks Release link imports and minos without executing probes. See `PLAN_IOS_PROXIMITYREADER.md` and `PLAN_VALIDATION_IOS_PROXIMITYREADER.md`

Apple describes the property as support for the current device model (iPhone XS or newer), not Tap to Pay readiness. It does not inspect OS compatibility, entitlement, merchant/account, region, payment service provider, reader preparation, NFC access, or transaction status. A scalar C output is safe: no asynchronous result, retained state, native object, or caller input pointer exists

Primary sources: [Apple `PaymentCardReader.isSupported`](https://developer.apple.com/documentation/proximityreader/paymentcardreader/issupported) and [Apple Tap to Pay integration requirements](https://developer.apple.com/documentation/proximityreader/adding-support-for-tap-to-pay-on-iphone-to-your-app)

## Proposed feature, symbol, contract

- Cargo feature: `ios-proximity-reader`, backed by an optional target-iOS dependency `ios-proximity-reader`
- Header: `framework_ios_proximity_reader.h`
- Export: `FrameworkStatus framework_ios_proximity_reader_tap_to_pay_device_model_supported(FrameworkIosProximityReaderBoolean *out_supported)`
- C Boolean: `typedef uint8_t FrameworkIosProximityReaderBoolean`; write exactly 0 or 1 only on `FRAMEWORK_STATUS_OK`
- Output: required caller-owned valid, properly aligned writable byte for the full synchronous call; caller prevents unsynchronized access; initialize to 0 before work; do not retain the output address
- Status: null output => `FRAMEWORK_STATUS_INVALID_ARGUMENT`; valid non-iOS call => `FRAMEWORK_STATUS_UNSUPPORTED` with output 0; on supported iOS targets, `Some(bool)` => `FRAMEWORK_STATUS_OK`; a defensive unexpected `None` from the iOS backend => `FRAMEWORK_STATUS_UNAVAILABLE`; caught Rust panic => `FRAMEWORK_STATUS_PANIC` with output 0. No native error path exists
- Threading: synchronous; caller prevents unsynchronized access to output; no main-thread rule or thread-safety promise is added
- Deployment: iOS device and Simulator link probes both use minos 15.4. Do not claim runtime fallback below 15.4; the native API floor is reflected in the deployment minimum
- Limits: device-model predicate only. A true result does not mean Tap to Pay is configured or usable. No entitlement, payment readiness, merchant/provider, region, account, NFC, session, UI, transaction, or runtime-result claim

## ABI manifest entry for root integration

Use `.optional_capabilities.ios_proximity_reader` with feature `ios-proximity-reader`, header and sole symbol above, API text naming `PaymentCardReader.isSupported` and iOS 15.4, and link probe minimums `aarch64-apple-ios = 15.4` and `aarch64-apple-ios-sim = 15.4`. Expected C imports are `ProximityReader` and `libSystem.B.dylib`; expected C++ imports add `libc++.1.dylib`. Record the one-byte output ownership, status mapping, no-main-thread/no-thread-safety contract, public Swift getter symbols, and the no-Swift-runtime/no-Objective-C-import boundary. Do not add a `FrameworkOwnedBuffer` creator

## Gate and root integration

The passing focused gate checks default/iOS/host feature trees, host unsupported stubs and imports, host/device/Simulator check and strict Clippy, Release static libraries, C11/C++17 compile/link, exact ProximityReader/libSystem imports, the two public Swift symbol imports, absence of Swift/Objective-C runtime libraries, the sole F18 archive export, and device/Simulator minos 15.4. Probes are build-only and were not executed. No tests ran

Root integration is complete: Cargo feature/dependency and module exports, ABI manifest, Cargo.lock, macOS CI, `PLAN_BINDINGS.md`, and the documentation index include F18
