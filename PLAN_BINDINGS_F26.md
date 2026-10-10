# PLAN_BINDINGS_F26.md — F26: default video-device C status query

## Objective

Expose one opt-in C Boolean snapshot over D57/B62's
`ios_camera_device_status::has_default_video_capture_device()`. Do not expose camera authorization,
capture setup, session lifecycle, or media access, and do not add a portable capability contract

## Backend and API bounds

B62 calls only `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)` through the typed
`objc2-av-foundation` 0.3.2 bindings with the `AVCaptureDevice` and `AVMediaFormat` features. The API
floor is iOS 4.0, from the public AVFoundation declarations; B62 guards that floor at runtime. The
Rust arm64 device link probe uses minos 10.0 and the Simulator probe uses minos 14.0. These probe
settings are distinct from the API floor. No camera permission request, Swift source, entitlement,
or new portable contract is added

`true` means AVFoundation returned a current default video device. It does not mean the app is
authorized to use the camera, the device is ready for capture, a session is configured, or a future
query will return the same value. The query does not call `requestAccessForMediaType:`, create
`AVCaptureDeviceInput` or `AVCaptureSession`, scan devices, read media, or show UI. If the host app
later requests camera access or creates an input, it must supply `NSCameraUsageDescription`. This
API adds no queue guarantee

## Exact C contract

- Optional Cargo feature: `ios-camera-device-status`
- Header: `framework_ios_camera_device_status.h`
- Export: `FrameworkStatus framework_ios_camera_device_status_has_default_video_capture_device(uint8_t *out_present)`
- `out_present` is required, writable for one byte, not retained, and initialized to zero before platform handling; the API checks only nullness, so a non-null pointer must address valid, properly aligned writable memory for the synchronous call, and the caller must prevent unsynchronized concurrent access
- On iOS success returns `OK` and writes exactly `0` or `1`
- A valid non-iOS call returns `UNSUPPORTED` with output zero
- Null output returns `INVALID_ARGUMENT` without a write; a caught panic returns `PANIC` with zero output
- No pointer, AVFoundation object, authorization state, capture session, or media crosses C
- The runtime B62 API floor is iOS 4.0; F26 link probes use device minos 10.0 and Simulator minos 14.0

## F26-owned files

- `bindings/c/src/ios_camera_device_status.rs`
- `bindings/c/include/framework_ios_camera_device_status.h`
- `bindings/c/check-ios-camera-device-status.sh`
- `bindings/c/check-ios-camera-device-status-link.sh`
- `docs/bindings/ios-camera-device-status.md`
- this plan

Root integrated the target-iOS optional `ios-camera-device-status` dependency and feature edge,
source module and re-export, ABI manifest entry and panic-boundary symbol, Cargo.lock resolution,
both macOS CI gates, aggregate plan evidence, and documentation index links

## Status and evidence

F26 is integrated in the root checkout. `sh bindings/c/check-ios-camera-device-status.sh` and
`sh bindings/c/check-ios-camera-device-status-link.sh` passed after root wiring. Host/device/Simulator
feature isolation, strict Clippy, Release archives, C11/C++17 links, exact AVFoundation/libSystem/libobjc
device and Simulator imports, symbol/string filters, and minos 10.0/14.0 passed. Host C imported only
libSystem; host C++ also imported libc++. Foundation was removed by dead stripping. Both gates are
wired in macOS CI; no passing workflow run had been recorded at this validation point. Local
validation used Xcode 26.6 build 17F113
and iOS/iOS Simulator SDK 26.5; no linked consumers/probes were executed. No Rust tests or live camera
query ran

Mainline run
[38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
later passed the F26 C ABI gates as part of steps 235–275 on macOS 15 and Xcode 27. Linked
probes were not executed; this is not runtime or device evidence.

The integrated static gate also asserts source, header, guide, plan, and manifest output-pointer
preconditions: valid aligned writable storage for the full synchronous call, caller protection from
unsynchronized access, zero initialization before platform handling, nullness-only validation, and
no pointer retention. It does not attempt to prove arbitrary C memory validity

- `sh bindings/c/check-ios-camera-device-status-link.sh` passed, including the static gate, locked host/device/Simulator checks, strict Clippy, Release archives, and C11/C++17 consumer links
- Xcode 26.6 build 17F113; iOS SDK 26.5; iOS Simulator SDK 26.5
- Device C and C++ direct imports: `AVFoundation`, `libSystem.B.dylib`, `libobjc.A.dylib`
- Simulator C and C++ direct imports: `AVFoundation`, `libSystem.B.dylib`, `libobjc.A.dylib`
- Host C direct import: `libSystem.B.dylib`; host C++ direct imports: `libSystem.B.dylib`, `libc++.1.dylib`; no AVFoundation, Objective-C, or Swift import in host archives/consumers
- `nm -u` and string inspection found `_objc_getClass`, `_objc_msgSend`, `_AVMediaTypeVideo`, `AVCaptureDevice`, and `defaultDeviceWithMediaType:`; no authorization, capture/session/input, or Swift symbol/string matched the forbidden set
- Device `LC_VERSION_MIN_IPHONEOS` version: 10.0; Simulator `LC_BUILD_VERSION` minos: 14.0
- The link command names Foundation, but `-Wl,-dead_strip_dylibs` removes it from the measured direct imports
- B62 runtime-guards the iOS 4.0 API floor; F26 link minos 10.0/14.0 are target settings, not API floors
- All consumer and probe binaries were built and inspected only; none were executed. No Rust tests or live camera query ran

These results apply to the installed Xcode 26.6/SDK 26.5 toolchain only. They do not establish runtime behavior on an iOS device or Simulator, a physical default camera, authorization, capture readiness, session behavior, or media access

## Acceptance and limits

- Static gate checks source/header symbol agreement, manifest contract, output-pointer preconditions
  across source/header/guide/plan/manifest, Rust formatting, shell syntax, whitespace, and standalone
  C11/C++17 header compilation
- Link/import gate checks feature isolation, host/device/Simulator build and strict Clippy, Release
  archives, C11/C++17 consumer links, F26 symbol parity, imports, forbidden capture/authorization
  symbols, and device/Simulator deployment metadata
- B62 API floor is iOS 4.0; target minima 10.0/14.0 are link settings
- Link probes establish linkage shape only. They do not establish a physical default device, camera
  authorization, capture readiness, session behavior, media access, or runtime availability
