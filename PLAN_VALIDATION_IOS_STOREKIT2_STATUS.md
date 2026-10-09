# G57 — StoreKit 2 status validation

The B63 gate proves a target build/link path and a Swift ABI call shape only. It does not read live payment status or test purchase behavior

## Commands

```sh
sh platform/ios/ios-storekit2-status/check.sh
```

The script runs host, device, and Simulator `cargo check`; strict Clippy; rustdoc; rustfmt; a transient Swift IR oracle; device and Simulator link/import inspection; `cargo xtask docs-check`; `cargo xtask zero-swift-source`; and `git diff --check`. It does not add or run tests

## Swift ABI criterion

The oracle uses the installed StoreKit SDK interface and emits IR for `arm64-apple-ios15.0` and `arm64-apple-ios15.0-simulator`. Both Swift IR files must call `_$s8StoreKit03AppA0O15canMakePaymentsSbvgZ` as `swiftcc i1 ()`. Clang C IR for the bridge must match that call shape, and both StoreKit.tbd files must export the symbol

## Link criterion

Each Rust-to-C link probe must import only `StoreKit` and `libSystem.B.dylib`, with StoreKit weak. `nm -u` must retain the exact Swift symbol. The API/symbol availability floor is iOS 15.0; link probes use Rust target defaults of device minos 10.0 and Simulator minos 14.0 to inspect weak-link compatibility below that floor. The current artifact audit used `nm -m` to confirm an undefined weak external and Clang `-O3` IR/arm64 assembly to confirm an explicit null check before the `swiftcall`; the persistent link script does not run the probe or claim older-OS runtime behavior. Direct `libswiftCore`, Swift retain/release, or any Swift runtime import fails the gate

## Exclusions

- No probe execution, live payment check, account access, product lookup, transaction, or payment UI
- No claim that the installed Xcode 26.6 toolchain meets the Xcode 27.x plan baseline
- No runtime proof for the absent-symbol fallback or any iOS version below 15.0

## Status

Passed in the isolated worktree and integrated checkout on Rust 1.94.1, Xcode 26.6 build 17F113,
Swift 6.3.3, Apple Clang 21.0.0, and iOS SDK 26.5. The Swift and C compiler oracles matched
`swiftcc i1 ()`; StoreKit.framework and the exact property symbol were weak imports, and the C
bridge null-checked the symbol before call. Device probe minos was 10.0; Simulator probe minos was
14.0. Release probes were built and inspected, not executed. The weak fallback has no runtime
evidence below the iOS 15.0 API/symbol floor. This toolchain is below the planned Xcode 27.x
baseline. No tests or live payment query were run.
