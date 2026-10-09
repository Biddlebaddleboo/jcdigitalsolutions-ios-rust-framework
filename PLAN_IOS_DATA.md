# PLAN_IOS_DATA.md — Workstream B19: Explicit iOS CFData Copies

## Status

B19 source, docs, G13 CI gates, Cargo.lock/workspace integration, and shared index/matrix updates
are complete on Xcode 26.6 build 17F113, iOS SDK 26.5, and Rust 1.94.1. Workspace check, strict
Clippy, tests, rustdoc, no_std checks/link probe, docs/zero-Swift checks, and diff checks pass. No
B19 CFData runtime test, probe execution, live app use, runtime parity, allocation-under-pressure,
or performance evidence

## Objective

Add a small iOS adapter that copies between D13's portable `ByteView` / `OwnedBytes` values and
immutable Core Foundation `CFData`. Expose a borrowed `&CFData` handle for Apple APIs that accept
`CFDataRef` or its toll-free bridged `NSData` counterpart

## Dependencies

- D13 `framework-data` is integrated; see `PLAN_CAPABILITIES_DATA.md`
- The workspace already uses `objc2-core-foundation` 0.3.2 for iOS Keychain data
- The installed Xcode 26.6 / iOS SDK 26.5 declares `CFDataCreate`, `CFDataGetLength`, and
  `CFDataGetBytePtr` in `CoreFoundation/CFData.h` without an API availability annotation. Do not
  invent a minimum iOS version from these declarations

## Write scope

- `PLAN_IOS_DATA.md`
- `platform/ios/ios-data/**`
- `docs/ios/data.md`

Root owns workspace/lockfile reconciliation, CI wiring and validation workstream G13, the
capability manifest, shared indexes, and aggregate progress docs. Do not edit those shared files

## Portable/native API

- Add an `ios-data` package that depends on `framework-data` and the existing workspace
  `objc2-core-foundation` binding with only the required `alloc` and `CFData` features
- Add an immutable owner such as `IosData` around `CFRetained<CFData>`
- Provide `IosData::copy_from(ByteView<'_>) -> Result<IosData, IosDataError>` using the fallible
  `CFData::new` wrapper for `CFDataCreate`; do not use `CFData::from_bytes`, which panics on native
  allocation failure
- Provide `IosData::copy_to_owned(&self) -> Result<OwnedBytes, IosDataError>`; reserve Rust output
  storage with `try_reserve_exact` before copying and report allocation failure without panic
- Provide a borrowed `IosData::as_cf_data(&self) -> &CFData` handle whose lifetime is tied to the
  owner; do not expose a raw pointer or ownership-transfer API
- Keep length conversion checked against `CFIndex`; map native allocation failure, Rust reserve
  failure, and length overflow to documented error variants
- Keep `ios-data` source target-gated to iOS and do not require `Send`, an executor, dynamic
  dispatch, global state, or unsafe caller code
- Add a link/import probe for device and Simulator. It may reference the Core Foundation create,
  length, and byte access symbols but must not execute the probe

## Boundaries

- Both conversion directions are explicit O(n) copies. `CFDataCreate` copies its input bytes;
  copying back creates Rust-owned bytes
- Do not use `CFDataCreateWithBytesNoCopy`, `CFData::with_bytes_no_copy`, `CFData::from_static_bytes`,
  `CFMutableData`, or an ownership-transfer/zero-copy claim
- Do not add `CFString`, `NSString`, UTF-8 decoding, string normalization, data encoding, file I/O,
  persistence, Keychain policy, or a generic serialization layer
- No permission, entitlement, or Info.plist key is needed for this in-memory Core Foundation API
- CFData's `NSData` toll-free bridge is an Apple platform interoperability fact, not a claim that
  this workstream implements a Foundation wrapper
- Compile/link/import evidence does not establish live app use, runtime parity, or performance

## Apple API basis

Apple documents `CFDataCreate` as creating immutable data by copying the supplied byte buffer,
`CFDataGetLength` as returning its byte count, and `CFDataGetBytePtr` as returning read-only bytes.
Apple lists `CFDataRef` / `NSData` as toll-free bridged. The installed SDK declarations are in
`System/Library/Frameworks/CoreFoundation.framework/Headers/CFData.h`. See Apple's
[CFData documentation](https://developer.apple.com/documentation/corefoundation/cfdata),
[`CFDataCreate`](https://developer.apple.com/documentation/corefoundation/cfdatacreate%28_%3A_%3A_%3A%29?language=objc),
and [toll-free bridged types](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFDesignConcepts/Articles/tollFreeBridgedTypes.html)

## Validation and handoff

- Run formatting, host workspace checks, strict all-target Clippy, package rustdoc, device and
  Simulator `cargo check` / strict Clippy, the B19 import script, `cargo xtask docs-check`,
  `cargo xtask zero-swift-source`, and `git diff --check` after root integration
- Inspect the public API for `CFMutableData`, no-copy storage, raw pointer escape, `CFString`,
  panic-on-allocation paths, or performance claims
- Report exact API names, allocation/error behavior, linked imports, tested target/toolchain, and
  the absence of runtime/performance evidence

## Validation record

The installed Xcode 26.6 (build 17F113) / iOS SDK 26.5 header at
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/CoreFoundation.framework/Headers/CFData.h`
declares `CFDataCreate`, `CFDataGetLength`, `CFDataGetBytePtr`, and `CFDataGetBytes` without an
availability annotation. The wrapper uses `objc2-core-foundation` 0.3.2 `CFData::new` for the
fallible `CFDataCreate` call, then `CFData::length` and `CFData::bytes` for the Rust copy-back

On Rust 1.94.1, these commands passed

- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-data/Cargo.toml -- --check`
- `cargo +1.94.1 check --locked --offline -p ios-data`
- `cargo +1.94.1 check --locked --offline -p ios-data --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-data --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-data -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-data --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-data --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 clippy --workspace --all-targets --locked --offline -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p ios-data --no-deps`
- `sh platform/ios/ios-data/check-link-imports.sh`
- `cargo +1.94.1 --locked --offline xtask docs-check`
- `cargo +1.94.1 --locked --offline xtask zero-swift-source`
- `sh -n platform/ios/ios-data/check-link-imports.sh`
- `git diff --check`

The device and Simulator probes link only `CoreFoundation` and `libSystem.B.dylib` as direct
libraries. The script requires undefined references to `_CFDataCreate`, `_CFDataGetBytes`, and
`_CFDataGetLength`. `nm -u` also shows `CFRelease` and system/runtime symbols; it shows no
Swift/Python runtime, Objective-C class/meta-class, Security, mutable/no-copy data, or string
symbols. `vtool` reports device `LC_VERSION_MIN_IPHONEOS` 10.0 and
Simulator `minos` 14.0, both with SDK 26.5; these artifact values do not define a CFData API floor.
Neither probe was executed. No runtime, allocation-pressure, app integration, parity, or performance
behavior was verified
