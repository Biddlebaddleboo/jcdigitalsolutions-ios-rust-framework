# B455: Camera capture session and live preview

## Objective

Add an iOS camera session for capability row 046. The session uses the existing default video device, accepts one device input, and exposes a native preview layer for a host view. It adds live preview only; it does not produce photo, movie, or sample-buffer output

## Boundary

- Keep `D57`/`B62` as a point-in-time default-device query; do not change its semantics
- Reuse the existing camera authorization contract and `ios-media-authorization`; require status `Authorized` before input creation
- Never prompt as a side effect of session construction. The host may explicitly request camera access through the existing API and must provide `NSCameraUsageDescription`
- Use only the default `AVMediaTypeVideo` device; do not enumerate, select, persist, or monitor devices
- Configure one `AVCaptureSession` and one `AVCaptureDeviceInput`; expose an `AVCaptureVideoPreviewLayer` for native UIKit composition
- Configure the session graph on a private per-session serial `DispatchQueue`, then create and expose the preview layer on the main thread
- Keep `startRunning` and `stopRunning` off the main/UI thread because both calls block. Enqueue lifecycle work on the same serial queue; public start/stop requests must not block the caller
- Start/stop requests return no completion or runtime-ready result; they do not prove that capture started, stopped, or delivered preview
- Keep preview-layer access on the main thread. Do not claim Objective-C session types are `Send` or `Sync`; any private unsafe transfer must be guarded and justified by exclusive serial-queue access, while the public session remains `!Send`/`!Sync`
- Use only public Apple APIs. Add no Swift source, entitlement, C ABI, audio engine, photo/movie output, sample buffer, recording, or frame access
- Keep this as a separate iOS crate, `ios-camera-capture-session`; do not alter the portable authorization contract or the existing authorization/device-status package scopes

## API and availability

- Package: `platform/ios/ios-camera-capture-session`
- Crate: `ios-camera-capture-session`
- Public owner: `IosCameraCaptureSession`
- The session, input, and preview-layer APIs have an iOS 4.0 floor; `authorizationStatusForMediaType:` has an iOS 7.0 runtime floor, enforced by a fail-closed availability guard. The effective package/link floor is iOS 10.0 for device and iOS 14.0 for Simulator, as reported by the current target link probes; keep profile target minima aligned with those toolchain outputs
- Native frameworks: AVFoundation and QuartzCore; retain only additional imports required by the implementation and explain them in the package gate
- The host supplies `NSCameraUsageDescription`; no entitlement is required

## Acceptance

- Constructor errors distinguish missing authorization, missing default video device/media type, input creation failure, and session input rejection
- Session configuration and start/stop are ordered on one private serial queue; start/stop do not block the caller. Drop requests stop without claiming synchronous teardown or runtime success
- Host test `device_input_requires_granted_camera_authorization` permits only `Authorized` and rejects `NotDetermined`, `Restricted`, `Denied`, and `Unknown`
- The registered `ios-camera-capture-session` profile enforces source guards for authorization-before-input, the iOS 7.0 availability guard, the default-device/input/session/preview APIs, and absence of prompt, photo/movie/sample/audio, and enumeration APIs
- The profile selects device and Simulator compile/Clippy/rustdoc gates and target link/import checks
- Device and Simulator link probes verify only the required public Apple/system imports and run no capture code
- Record host compile/test, device compile/link, and Simulator compile/link as distinct evidence. Neither compile nor link checks prove preview delivery, authorization UI behavior, camera readiness, or device runtime behavior

## Final qualification

`ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-camera-capture-session` reports 51 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED

- Host test `device_input_requires_granted_camera_authorization` passes; the 17 registered positive and negative source guards pass
- iOS 10.0 device and iOS 14.0 Simulator compile/link/import checks pass
- The two skipped checks remain unverified. No authorization prompt, camera use, preview delivery, or device/Simulator runtime test ran

## Exclusions

- Photo capture, movie recording, audio capture, output objects, sample buffers, frame callbacks, and image processing
- Camera enumeration/selection, switching, torch, zoom, focus, orientation policy, session notifications, and recovery policy
- Authorization requests or UI initiated by the capture crate
- Portable camera-session API, C ABI, Python binding, or Swift shim
