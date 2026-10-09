# PLAN_IOS_MODELIO_STATUS.md — Workstream B67: iOS ModelIO Import-Extension Query

## Status

The iOS-only wrapper uses `objc2-model-io` 0.3.2 with default features off and only `MDLAsset`.
The safe Rust API converts the caller's `&str` to `NSString` and calls
`MDLAsset::canImportFileExtension`. It does not create an asset or access a URL or file data.

The package gate passed in a temporary isolated workspace with a matching local lockfile. It ran
format, host check, device and Simulator target checks, strict Clippy for both Apple targets, target
feature-tree checks, build-only device and Simulator link probes, and iOS rustdoc. The root lockfile
was not changed; rerun the locked gate after workspace integration refreshes that lockfile. No tests
were added or run.

Both link probes imported exactly `Foundation`, `ModelIO`, `libSystem.B.dylib`, and
`libobjc.A.dylib`; they contained `_objc_msgSend` and the `canImportFileExtension:` selector. Their
embedded deployment `minos` values were device 10.0 and Simulator 14.0. The probes were built and
inspected only, never executed.

## SDK and binding evidence

- Xcode 26.6 / iPhoneOS SDK 26.5 `ModelIO.framework/Headers/MDLAsset.h` declares `MDLAsset`
  available on iOS 9.0. `+[MDLAsset canImportFileExtension:]` has no method-level availability
  annotation, so the class floor applies.
- The method signature is `+ (BOOL)canImportFileExtension:(NSString *)extension`.
- `objc2-model-io` 0.3.2 has feature `MDLAsset` and declares
  `pub unsafe fn canImportFileExtension(extension: &NSString) -> bool`.
- `objc2-foundation::NSString::from_str(&str) -> Retained<NSString>` provides the owned immutable
  argument required for the duration of the class-method call.
- Device and Simulator target link probes use iOS 10.0 and 14.0 deployment floors, respectively.

Primary references:

- [Apple `canImportFileExtension(_:)` documentation](https://developer.apple.com/documentation/modelio/mdlasset/canimportfileextension%28_%3A%29)
- [Apple `MDLAsset` documentation](https://developer.apple.com/documentation/modelio/mdlasset)
- [`objc2-model-io` 0.3.2 `MDLAsset` binding](https://docs.rs/objc2-model-io/0.3.2/objc2_model_io/struct.MDLAsset.html#method.canImportFileExtension)
- [`objc2-foundation` 0.3.2 `NSString` binding](https://docs.rs/objc2-foundation/0.3.2/objc2_foundation/struct.NSString.html#method.from_str)

## Scope

- Add `platform/ios/ios-modelio-status` as a target-gated, no-std package with no non-iOS stub.
- Expose only `can_import_file_extension(&str) -> bool` on iOS.
- Keep `objc2-model-io` default features off and enable only `MDLAsset`; use only the Foundation
  `NSString` feature directly.
- Do not add or run tests, create/load assets, touch files, or make parsing, rendering, GPU, parity,
  or performance claims.
- Root Cargo files, Cargo.lock, CI, global capability manifests/counts, and shared docs indexes are
  orchestrator-owned; the root integration is tracked separately.

## Validation

Run `sh platform/ios/ios-modelio-status/check.sh` after workspace/lockfile integration. The gate formats,
checks, and strict-Clippy checks device and Simulator targets, confirms the `MDLAsset` feature
without ModelIO default features, builds but never runs link probes, and checks direct imports and
deployment metadata. See the package gate for exact commands.

## Limits

Probe compilation is not runtime behavior. No check proves that a specific extension is supported on
every OS/device, that a particular file parses, or that ModelIO can render it. The inspected host is
Xcode 26.6 / iOS SDK 26.5, below the planned Xcode 27.x baseline.
