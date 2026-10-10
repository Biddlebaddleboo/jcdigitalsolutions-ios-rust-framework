# D45 / B50 — VideoToolbox hardware-decode capability snapshot

## Scope

Implement one non-invasive status predicate for row 050: whether the current iOS system reports
hardware decode support for a caller-supplied `CMVideoCodecType` FourCC. The portable type is
framework-owned and uses four display-order bytes. The backend calls only
`VTIsHardwareDecodeSupported`.

This slice does not implement encoding, software-decoder support, decoder sessions, frame I/O,
asset loading, resource reservation, or a guarantee that a decoder will be available later. It
does not request permission or expose a native handle.

## API and bounds

- Public API: Apple VideoToolbox `VTIsHardwareDecodeSupported(CMVideoCodecType)`.
- iOS API floor: 11.0, checked at runtime with `objc2::available!(ios = 11.0, ..)`; earlier OS
  versions and non-iOS targets return `None`.
- `CMVideoCodecType` is a `FourCharCode`; the portable `VideoCodecType` maps four display-order
  bytes to the same big-endian `u32` value.
- Frameworks: VideoToolbox for the predicate and CoreMedia for the `CMVideoCodecType` alias; typed
  binding: `objc2-video-toolbox` 0.3.2, defaults disabled, with `VTDecompressionSession` and
  `objc2-core-media` features. No Swift ABI, privacy permission, Info.plist key, or entitlement is
  required for this scalar query.
- Keep the `ios-media` Cargo feature `videotoolbox` opt-in and disabled by default so CoreMedia and
  AVAudioSession-only consumers do not link VideoToolbox.
- The status is capability metadata only. Apple states that `true` does not guarantee future
  hardware decoder resources.

## Status and evidence

Status: bounded implementation complete; row 050 remains partial support, not full VideoToolbox
encode/decode support.

The installed Xcode 26.6 / iPhoneOS 26.5 SDK header
`VideoToolbox.framework/Headers/VTDecompressionSession.h` marks the function `API_AVAILABLE(ios(11.0))`.
`objc2-video-toolbox` 0.3.2 exposes the typed `VTIsHardwareDecodeSupported(CMVideoCodecType)`
binding behind the listed feature gates. Apple documentation:
[VideoToolbox predicate](https://developer.apple.com/documentation/videotoolbox/vtishardwaredecodesupported%28_%3A%29),
[CoreMedia codec type](https://developer.apple.com/documentation/coremedia/cmvideocodectype).

The package gate `sh platform/ios/ios-media/check.sh` passed on Xcode 26.6 (build 17F113), iPhoneOS
SDK 26.5. It runs format checks, locked portable and host checks, iOS device and simulator
`cargo check`, strict library Clippy, docs, and a feature-surface check that rejects VideoToolbox
defaults, encoder, or encoder-list features. `cargo xtask docs-check` passed, and the current manifest
summary counts match all 113 rows (36 implemented portable contracts, 22 partial portable contracts,
85 partial implementations, 85 `B`, 28 `X`). `git diff --check` passed. No tests, link probes,
probe execution, live codec query, or device behavior validation is included.

Gate recheck (2026-10-10): `sh platform/ios/ios-media/check.sh` PASS at source-equivalent
commit `3900cd1`; later commits through `2a38984` are docs/CI-only for these paths. Env: Xcode 26.6
(build 17F113), SDK 26.5, Rust/Cargo 1.94.1, tools 0.1.0. Opt-in feature-isolation check: PASS;
iOS API/device floor 11.0, Simulator build floor 14.0. No tests, link probes, runtime codec query,
or device behavior check ran
