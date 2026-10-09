# iOS default video-device C API

The opt-in `ios-camera-device-status` feature exposes one Boolean snapshot over B62's AVFoundation
default-video-device query. It reports only whether AVFoundation returns a current default video
device. It does not report camera authorization, capture readiness, session state, or future
availability

## Call contract

`framework_ios_camera_device_status_has_default_video_capture_device` takes one required writable
output byte. The API checks only nullness; any non-null pointer must actually address valid,
properly aligned writable memory for the synchronous call, and the caller must prevent
unsynchronized concurrent access to that byte. The wrapper initializes it to zero before platform
handling. On iOS, `OK` writes
exactly `0` or `1`; a valid non-iOS call returns `UNSUPPORTED` with zero output. A null pointer
returns `INVALID_ARGUMENT` without a write, and a caught Rust panic returns `PANIC` with zero
output. No pointer or native object is retained or exposed

The backend calls only
`AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`. It does not request authorization,
create `AVCaptureDeviceInput` or `AVCaptureSession`, scan devices, read media, or show UI. If the
host app later requests camera access or creates an input, it must supply `NSCameraUsageDescription`.
This query does not establish that access or capture will succeed. No queue guarantee is added

The API floor is iOS 4.0. F26 device link probes use minos 10.0 and Simulator probes use minos 14.0;
these are link settings, not the API floor. No new portable capability contract is introduced

Link probes validate imports and deployment metadata only. They do not run the query, prove that a
physical camera exists, or test authorization/capture behavior. Consumers and probes are linked and
inspected, never executed

## Link evidence

On Xcode 26.6 build 17F113 with iOS and Simulator SDK 26.5, device and Simulator C11/C++17 probe
imports were `AVFoundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`. Host C imported only
`libSystem.B.dylib`; host C++ imported `libSystem.B.dylib` and `libc++.1.dylib`. Device minos was
10.0 and Simulator minos was 14.0. The iOS 4.0 API floor remains distinct from those link settings.
The probes were built and inspected, never executed

See [F26 binding plan](../../PLAN_BINDINGS_F26.md), [B62 backend plan](../../PLAN_IOS_CAMERA_DEVICE_STATUS.md),
and the D57 camera capability contract in [PLAN_CAPABILITIES.md](../../PLAN_CAPABILITIES.md)
