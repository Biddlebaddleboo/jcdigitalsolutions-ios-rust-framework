# PLAN_BINDINGS_F25.md — F25: VideoToolbox hardware-decode C query

## Objective

Expose one opt-in C status query over D45/B50's `VTIsHardwareDecodeSupported` predicate. Do not add
decoder-session creation, frame processing, encode support, or a resource guarantee

## Backend and API bounds

B50 exposes `ios_media::hardware_decode_support(VideoCodecType)`. The portable codec value stores
four display-order FourCC bytes as a big-endian `u32`; the C API takes that same `uint32_t` value and
does not whitelist codec identifiers. B50 calls only `VTIsHardwareDecodeSupported`. The API floor
is iOS 11.0; its runtime availability guard returns `None` below that floor and on non-iOS targets.
No permission, usage-description key, or entitlement is required

The API is synchronous and adds no thread-affinity or thread-safety promise. It does not create a
decoder session, process media, report software decoding, reserve hardware, or guarantee a future
decoder resource

## Exact C contract

- Optional Cargo feature: `ios-videotoolbox`; it enables `ios-media` with default features disabled
  and the package's opt-in `videotoolbox` feature
- Header: `framework_ios_videotoolbox.h`
- Export: `FrameworkStatus framework_ios_videotoolbox_hardware_decode_supported(uint32_t codec_fourcc, uint8_t *out_supported)`
- `codec_fourcc` is the four display-order bytes in a big-endian `uint32_t`; e.g. `avc1` is `0x61766331`. Every 32-bit value is passed through
- When non-null, `out_supported` must address one valid, properly aligned writable byte for the full synchronous call. The caller must prevent unsynchronized concurrent access. The API checks only nullness and does not retain the output pointer. The output is initialized to zero before platform handling
- On iOS 11.0 or later, `OK` writes exactly `0` or `1`
- If B50's runtime guard reports the API unavailable, `UNAVAILABLE` leaves output zero; a valid non-iOS call returns `UNSUPPORTED` and leaves output zero
- A null output pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; a caught panic returns `PANIC` with zero output
- No pointer, codec object, decoder, or session crosses C

## F25-owned files

- `bindings/c/src/ios_videotoolbox.rs`
- `bindings/c/include/framework_ios_videotoolbox.h`
- `bindings/c/check-ios-videotoolbox.sh`
- `bindings/c/check-ios-videotoolbox-link.sh`
- `docs/bindings/ios-videotoolbox.md`
- this plan

Root integrated the optional `framework-media` and target-iOS `ios-media` dependencies, the
`ios-videotoolbox` feature, source module and re-export, ABI manifest entry, panic-boundary symbol
list, Cargo.lock resolution, both macOS CI gates, aggregate plan evidence, and documentation index
links

## Status and evidence

F25 is integrated in the root checkout. `sh bindings/c/check-ios-videotoolbox.sh` passed Rust
formatting, both shell syntax checks, ABI manifest syntax/contract, source/header symbol agreement,
whitespace, and standalone C11/C++17 header compilation. `sh bindings/c/check-ios-videotoolbox-link.sh`
passed again after root integration. Both gates are wired in macOS CI; no passing workflow run is
recorded. Local validation used Xcode 26.6 (build 17F113), iPhoneOS SDK 26.5, and iPhoneSimulator
SDK 26.5; this remains below the planned Xcode 27.x baseline

`sh bindings/c/check-ios-videotoolbox-link.sh` passed host/device/Simulator feature-tree isolation,
locked host/device/Simulator checks, strict Clippy, Release archives, and host/device/Simulator
C11/C++17 links. All linked probes imported only `VideoToolbox` and `libSystem.B.dylib`; the target
undefined-symbol inspection found `_VTIsHardwareDecodeSupported` and no VideoToolbox session,
encoding, AVFAudio, Objective-C messaging, or Swift symbol. Host C linked with only
`libSystem.B.dylib`; host C++ linked with `libSystem.B.dylib` and `libc++.1.dylib`. Inspected device
minos was 11.0 and Simulator minos was 14.0. A separate minos-10.0 device link experiment found a
strong unresolved `_VTIsHardwareDecodeSupported` symbol, not a weak import. F25 therefore uses
device minos 11.0, matching the API floor, and makes no pre-floor link/runtime claim. The link
command removes unused dylibs, so CoreMedia and AVFAudio are not direct imports of this C probe

After the null-rejection/output-zeroing order check was added, root reran both
`sh bindings/c/check-ios-videotoolbox.sh` and `sh bindings/c/check-ios-videotoolbox-link.sh`. The
static source/header/guide/plan pointer assertions, host/device/Simulator checks and strict Clippy,
Release archives, C11/C++17 links, exact VideoToolbox/libSystem imports, export parity, and minos
11.0/14.0 passed. No tests, consumers, probes, or live VideoToolbox queries ran

The enabled Cargo target graph includes `objc2-avf-audio` because the existing `ios-media` package
also contains B22's AVAudioSession backend. The linked F25 C/C++ probes did not import AVFAudio or
its symbols. No tests, consumers, probe binaries, or live VideoToolbox query were executed; the
link results establish symbol/import/deployment shape only. B50's iOS API floor remains 11.0;
Simulator minos 14.0 is distinct from that API floor

The iOS feature graph uses `objc2-video-toolbox` 0.3.2 with defaults disabled and the existing
`VTDecompressionSession` plus `objc2-core-media` binding features required by `ios-media`. The
host feature graph includes the portable `framework-media` contract but excludes `ios-media` and
the Apple bindings; the default iOS graph excludes both F25 backend dependencies. The linked C and
C++ probes imported only VideoToolbox and libSystem on device/Simulator; host C imported only
libSystem and host C++ also imported libc++

Gate recheck (2026-10-10): `sh bindings/c/check-ios-videotoolbox-link.sh` PASS at
source-equivalent commit `3900cd1`; later commits through `2a38984` are docs/CI-only for these paths. Env:
Xcode 26.6 (build 17F113), SDK 26.5, Rust/Cargo 1.94.1, tools 0.1.0. Feature isolation + exact
linked-import audit: PASS; device minos 11.0, Simulator minos 14.0. Scope: C11/C++17 link/import
compile only. No tests, consumers, probe binaries, or runtime codec query ran

## Acceptance and limits

- Static gate checks source/header symbol agreement, manifest contract when root wiring exists,
  Rust formatting, shell syntax, whitespace, and standalone C11/C++17 header compilation
- After root integration, the static gate also asserts a null check before the first output write,
  output zeroing before platform handling, and the valid/aligned/writable full-call, caller-sync,
  nullness-only, and no-retention contract across source, header, guide, and plan; these checks pass
- Link/import gate checks feature isolation, host/device/Simulator build and strict Clippy, Release
  archives, C11/C++17 consumer links, F25 symbol parity, imports, forbidden out-of-scope symbols,
  and device/Simulator deployment metadata
- Device link minimum is 11.0, matching the API floor; Simulator minimum is 14.0 and is only a link
  setting. The below-floor status branch is not established as a safely loadable linked binary
- Link probes validate linkage shape only. They do not establish a live support value, codec
  decoding, decoder-session creation, media parity, or performance
