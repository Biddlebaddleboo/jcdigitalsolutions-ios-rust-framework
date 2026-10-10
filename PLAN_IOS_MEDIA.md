# PLAN_IOS_MEDIA.md — Workstream B22: CoreMedia Finite Time Value

## Status

B22 constructs the public `CMTime` value directly to avoid the strong `libswiftCoreMedia.dylib` load caused by `CMTimeMake` at the iOS 12.0 device floor. The package check and strict link/import gate pass locally on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5; the Xcode 27 CI lane remains required. Probe binaries are not run. No runtime media claim

## Objective

Map D17's finite rational media time to the native CoreMedia `CMTime` struct by value

## Dependencies

- D17 `framework-media::MediaTime`
- exact `objc2-core-media` 0.3.2 pin with only the `CMTime` feature; its public `CMTime` fields and `CMTimeFlags::Valid` define the native value layout
- Xcode 26.6 / iOS SDK 26.5 CoreMedia header declares `CMTime` and `CMTimeMake` at iOS 4.0; the `CMTimeMake` wrapper imports a strong Swift CoreMedia runtime dylib at the iOS 12.0 deployment floor

## Write scope

- `PLAN_IOS_MEDIA.md`
- `platform/ios/ios-media/**`
- `docs/ios/media.md`

Root owns workspace and lock integration, CI, capability manifest and counts, shared indexes, and aggregate validation docs. Do not edit those shared paths

## API

- `IosMediaTime::from_portable(MediaTime)` constructs the pinned crate's public `CMTime` fields directly: the exact numerator, the strictly positive timescale, `CMTimeFlags::Valid`, and epoch zero
- The literal has the same finite, exact-rational semantics as `CMTimeMake`; it makes no foreign call and does not require `libswiftCoreMedia.dylib`
- No new value check is needed; D17 guarantees the strictly positive timescale required by CoreMedia
- `IosMediaTime::as_cm_time(self) -> CMTime` returns a copy by value
- `CMTime` flags denote valid, exact rational time; epoch is zero; `HasBeenRounded` and implied-value flags are clear
- No object owner, raw pointer, `CMTimeRange`, `CMSampleBuffer`, AVFoundation, executor, permission, entitlement, or Info.plist key
- The SDK header and crate docs mark `CMTime` and `CMTimeFlags` as iOS 4.0+. The build probe uses min iOS 12.0 and Simulator 14.0 as toolchain targets only

## Layout and link evidence

- The pinned Rust crate has `CMTime` with `repr(C, packed(4))`; const assertions check size 24, align 4, and field offsets 0, 8, 12, and 16
- `examples/cm_time_layout.c` checks the public CoreMedia header with C11 `_Static_assert` for the same layout on device and Simulator targets
- `check-link-imports.sh` links with `-Wl,-dead_strip_dylibs`, builds but never runs the Rust probe, checks the C header layout, exact remaining direct import `libSystem.B.dylib`, rejects `CMTimeMake`, Swift dylibs/symbols and forbidden out-of-scope media/runtime symbols, and checks target minos
- No Swift source or additional native source

## Validation

- Run package format, host/device/Simulator check, strict Clippy, rustdoc, and the link/import script
- Run `sh -n platform/ios/ios-media/check-link-imports.sh` and `git diff --check`
- Do not claim that link or layout evidence proves live CoreMedia or AVFoundation use, media playback, capture, runtime parity, or performance

## Apple API basis

The SDK file is `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/CoreMedia.framework/Headers/CMTime.h`. It uses `#pragma pack(push, 4)`, defines `CMTime` as `int64_t value`, `int32_t timescale`, `uint32_t flags`, and `int64_t epoch`, and marks the type and `CMTimeMake` iOS 4.0+. The header states that timescales must be positive, `kCMTimeFlags_Valid` must be set, and `CMTimeMake` initializes the value/timescale fields with epoch implied to zero. A portable `MediaTime` has a positive timescale and no invalid, infinite, indefinite, or rounded state, so the direct value uses only the valid flag

The exact 0.3.2 crate pin exposes `objc2_core_media::CMTime` and `CMTimeFlags::Valid` behind feature `CMTime`; its public fields are `repr(C, packed(4))`. `IosMediaTime::from_portable` uses those public fields after D17's positive-timescale type check. See [objc2-core-media 0.3.2 `CMTime`](https://docs.rs/objc2-core-media/0.3.2/objc2_core_media/struct.CMTime.html) and [Apple `CMTimeMake`](https://developer.apple.com/documentation/coremedia/cmtimemake(_:_:))

## Validation record

On Rust/Cargo 1.94.1 / Xcode 26.6 build 17F113 / iOS SDK 26.5, these checks pass after the literal-value change

- `sh platform/ios/ios-media/check.sh` (format, host/device/Simulator checks, strict Clippy, rustdoc, and feature isolation)
- `sh -n platform/ios/ios-media/check-link-imports.sh`
- `sh platform/ios/ios-media/check-link-imports.sh`
- `git diff --check`

The refreshed device and Simulator link probes import only `libSystem.B.dylib`; `CoreMedia` is stripped because the literal path imports no CoreMedia function. `nm -u` has no `_CMTimeMake` or Swift runtime symbol. The strong `@rpath/libswiftCoreMedia.dylib` dependency from the prior Xcode 27 probe is absent. `vtool` reports device minos 12.0 and Simulator minos 14.0 on SDK 26.5. C11 header layout asserts and Rust const layout asserts pass on both targets. Neither probe runs. Xcode 26.6 is below the Xcode 27.x CI baseline. No app runtime, media parity, or performance evidence
