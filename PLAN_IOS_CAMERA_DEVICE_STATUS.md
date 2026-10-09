# B62: Default camera device query

## Purpose

B62 adds a small AVFoundation backend for D57 row 046. It exposes one synchronous, point-in-time query and does not add capture behavior

## API

- Crate: `ios-camera-device-status`
- Rust API: `ios_camera_device_status::has_default_video_capture_device() -> bool`
- Native call: `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`
- iOS floor: 4.0, from the `AVCaptureDevice` class and `AVMediaTypeVideo` declarations in the iOS SDK
- Framework: `AVFoundation.framework`
- objc2 feature set: `AVCaptureDevice` and `AVMediaFormat`

## Semantics and limits

- `true` means the native method returned a current default video device
- `false` means no default device, a non-iOS target, or a runtime below iOS 4.0
- The result does not report authorization, capture readiness, configured session state, or future availability
- The backend does not call `requestAccessForMediaType:`, create `AVCaptureDeviceInput` or `AVCaptureSession`, scan devices, read media, or present UI
- The host app must provide `NSCameraUsageDescription` before it requests camera access or creates an input
- The SDK method declaration has no main-thread annotation; this plan sets no queue guarantee
- No entitlement, permission prompt, callback, Swift source, or portable contract is part of this slice

## Link evidence

`check-link-imports.sh` builds one Rust probe for `aarch64-apple-ios` and one for `aarch64-apple-ios-sim`, inspects each binary with `otool -L`, `nm -u`, and `strings`, and requires only AVFoundation, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. The probes are not run

## Out of scope

- Camera or microphone authorization, owned by D31/B36
- No capture setup, input, session, output, movie file, photo, or media data access
- No camera device scan, choice, status watch, or saved state
- RoomPlan D56/B61
