# D57: Default video-device presence

## Scope

D57 adds one iOS-only status function for row 046, `ios_camera_device_status::has_default_video_capture_device() -> bool`. The function returns `true` if AVFoundation returns a current default device for `AVMediaTypeVideo`

This slice adds no portable camera contract. It does not close the full camera capture/session/device row

## Boundary

- Use only `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`
- Return `true` when the native call returns a device and `false` when it returns `nil`
- Keep `false` as the result on non-iOS targets and before the iOS 4.0 API floor
- Treat the value as a point-in-time hardware-presence hint only
- Keep camera authorization-status ownership in D31/B36

## Exclusions

- No capture input, session, output, sample, photo, or video flow
- No device enumeration or device selection
- No authorization query or request, prompt, UI, callback, or entitlement
- No claim of permission, capture readiness, future availability, or row-wide support
- No change to D31/B36 or RoomPlan D56/B61

## Acceptance

- Compile the package for host, iOS device, and iOS Simulator
- Pass strict Clippy, rustdoc, format, docs, zero-Swift, and diff checks
- Build but do not execute a device and Simulator link probe
- Inspect direct imports and require only AVFoundation, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`
- Add no portable contract or whole-row status claim; root integration may register the backend and
  source-verified metadata without changing B/X row counts. No tests, live camera access, or runtime
  parity are part of this scope

## Evidence

Apple's `AVCaptureDevice.h` declares `AVCaptureDevice` from iOS 4.0 and documents that `defaultDeviceWithMediaType:` returns the default device or `nil`. Apple's authorization guide states that first `AVCaptureDeviceInput` creation can display the permission alert; this slice creates no input and calls no authorization API

- [AVCaptureDevice](https://developer.apple.com/documentation/avfoundation/avcapturedevice)
- [defaultDeviceWithMediaType:](https://developer.apple.com/documentation/avfoundation/avcapturedevice/default%28for%3A%29)
- [Authorization guide](https://developer.apple.com/documentation/avfoundation/requesting-authorization-to-capture-and-save-media)
