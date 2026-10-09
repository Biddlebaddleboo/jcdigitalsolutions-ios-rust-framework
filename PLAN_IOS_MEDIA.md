# PLAN_IOS_MEDIA.md — Workstream B22: CoreMedia Finite Time Value

## Status

B22 source, guide, C header layout fixture, and local link/import script pass local checks on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5. The probe binaries are not run. No runtime media claim

## Objective

Map D17's finite rational media time to the native CoreMedia `CMTime` struct by value

## Dependencies

- D17 `framework-media::MediaTime`
- exact `objc2-core-media` 0.3.2 pin with only the `CMTime` feature
- Xcode 26.6 / iOS SDK 26.5 CoreMedia file declares `CMTime` and `CMTimeMake` at iOS 4.0; the 0.3.2 crate offers `CMTime::new`

## Write scope

- `PLAN_IOS_MEDIA.md`
- `platform/ios/ios-media/**`
- `docs/ios/media.md`

Root owns workspace and lock integration, CI, capability manifest and counts, shared indexes, and aggregate validation docs. Do not edit those shared paths

## API

- `IosMediaTime::from_portable(MediaTime)` calls `CMTime::new(value, timescale)` from the pinned `objc2-core-media` crate
- No new value check is needed; D17 guarantees the strictly positive timescale required by CoreMedia
- `IosMediaTime::as_cm_time(self) -> CMTime` returns a copy by value
- `CMTime` flags denote valid, exact rational time; epoch is zero
- No object owner, raw pointer, `CMTimeRange`, `CMSampleBuffer`, AVFoundation, executor, permission, entitlement, or Info.plist key
- The SDK header and crate docs mark the used type/constructor as iOS 4.0+. The build probe uses min iOS 12.0 and Simulator 14.0 as toolchain targets only

## Layout and link evidence

- The pinned Rust crate has `CMTime` with `repr(C, packed(4))`; const assertions check size 24, align 4, and field offsets 0, 8, 12, and 16
- `examples/cm_time_layout.c` checks the public CoreMedia header with C11 `_Static_assert` for the same layout on device and Simulator targets
- `check-link-imports.sh` links with `-Wl,-dead_strip_dylibs`, builds but never runs the Rust probe, checks the C header layout, exact direct imports `CoreMedia` and `libSystem.B.dylib`, `_CMTimeMake`, forbidden out-of-scope media/runtime symbols, and target minos
- No Swift source or additional native source

## Validation

- Run package format, host/device/Simulator check, strict Clippy, rustdoc, and the link/import script
- Run `sh -n platform/ios/ios-media/check-link-imports.sh` and `git diff --check`
- Do not claim that link or layout evidence proves live CoreMedia or AVFoundation use, media playback, capture, runtime parity, or performance

## Apple API basis

The SDK file is `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/CoreMedia.framework/Headers/CMTime.h`. It uses `#pragma pack(push, 4)`, defines `CMTime` as `int64_t value`, `int32_t timescale`, `uint32_t flags`, and `int64_t epoch`, and marks the type and `CMTimeMake` iOS 4.0+. It states that timescales must be positive

The exact 0.3.2 crate pin exposes `objc2_core_media::CMTime` behind feature `CMTime`; `IosMediaTime::from_portable` calls unsafe `CMTime::new` only after D17's positive-timescale type check. See [objc2-core-media 0.3.2 `CMTime`](https://docs.rs/objc2-core-media/0.3.2/objc2_core_media/struct.CMTime.html) and [Apple `CMTimeMake`](https://developer.apple.com/documentation/coremedia/cmtimemake(_:_:))

## Validation record

On Rust 1.94.1, these checks pass

- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-media/Cargo.toml -- --check`
- `cargo +1.94.1 check --locked --offline -p ios-media`
- `cargo +1.94.1 check --locked --offline -p ios-media --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-media --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-media -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-media --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-media --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p ios-media --no-deps`
- `sh -n platform/ios/ios-media/check-link-imports.sh`
- `sh platform/ios/ios-media/check-link-imports.sh`
- `git diff --check`

The device and Simulator link probes import exactly CoreMedia and `libSystem.B.dylib`; `-Wl,-dead_strip_dylibs` removes the unused CoreFoundation load command while `_CMTimeMake` remains imported from CoreMedia. `nm -u` shows `_CMTimeMake` and system symbols only. `vtool` reports device minos 12.0 and Simulator minos 14.0 with SDK 26.5. C11 header layout asserts and Rust const layout asserts pass on both targets. Neither probe runs. Xcode 26.6 build 17F113 / iOS SDK 26.5 is below the Xcode 27.x plan baseline. No app runtime, media parity, or performance evidence
