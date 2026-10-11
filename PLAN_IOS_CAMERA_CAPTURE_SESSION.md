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
- Use only public Apple APIs. B455 added no Swift source, entitlement, C ABI, audio engine, photo/movie output, sample buffer, recording, or frame access
- Keep this as a separate iOS crate, `ios-camera-capture-session`; do not alter the portable authorization contract or the existing authorization/device-status package scopes

## API and availability

- Package: `platform/ios/ios-camera-capture-session`
- Crate: `ios-camera-capture-session`
- Public owner: `IosCameraCaptureSession`
- The session, input, and preview-layer APIs have an iOS 4.0 floor; `authorizationStatusForMediaType:` has an iOS 7.0 runtime floor, enforced by a fail-closed availability guard. The effective package/link floor is iOS 10.0 for device and iOS 14.0 for Simulator, as reported by the current target link probes; keep profile target minima aligned with those toolchain outputs
- Native frameworks: AVFoundation and QuartzCore; B461 additionally imports CoreMedia and CoreFoundation for the public borrowed sample-buffer type, verified by the package link gate
- The host supplies `NSCameraUsageDescription`; no entitlement is required

## Acceptance

- Constructor errors distinguish missing authorization, missing default video device/media type, input creation failure, and session input rejection
- Session configuration and start/stop are ordered on one private serial queue; start/stop do not block the caller. Drop requests stop without claiming synchronous teardown or runtime success
- Host test `device_input_requires_granted_camera_authorization` permits only `Authorized` and rejects `NotDetermined`, `Restricted`, `Denied`, and `Unknown`
- The registered `ios-camera-capture-session` profile enforces source guards for authorization-before-input, the iOS 7.0 availability guard, the default-device/input/session/preview APIs, and absence of prompt, photo/movie/sample/audio, and enumeration APIs within B455 scope
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
- Camera enumeration/selection, switching, torch level control, zoom, focus, orientation policy, session notifications, and recovery policy; B473 adds only explicit torch on/off control
- Authorization requests or UI initiated by the capture crate
- Portable camera-session API, C ABI, Python binding, or Swift shim

## B461: video sample callback stream

B461 adds an opt-in, video-only `AVCaptureVideoDataOutput` callback to the existing session. It does not change B455's historical scope or add audio input, AVAudioSession mutation, Photos writes, device enumeration, private APIs, or Swift source

- `start_video_frame_stream` installs one callback per session. The callback receives a borrowed `&CMSampleBuffer` valid only for that call; the framework does not retain/copy samples. Caller retention is explicit and may increase memory use
- The callback runs on its own serial queue, separate from the capture-graph queue. `alwaysDiscardsLateVideoFrames` is explicitly `true`; callers must keep callbacks short. The framework cannot enforce a time bound on arbitrary callback code
- Explicit stop deactivates callback delivery before queueing detach. A claimed callback may finish; later queued callbacks observe inactivity. Delegate/output release and output removal occur after an asynchronous callback-queue barrier, never by synchronously waiting for user callback work
- Session stop/drop deactivates the stream and preserves callback/delegate lifetime through asynchronous drainage. Movie-file finalization behavior remains unchanged
- The package link gate now requires CoreMedia/CoreFoundation imports, the video-output/sample-buffer APIs, queue/delegate selectors, discard-late setting, output removal, and the existing iOS 10.0 device/iOS 14.0 Simulator deployment minima
- Registered profile command `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-camera-capture-session`: 87 PASS, 0 FAIL, 0 BLOCKED, 2 SKIPPED on Xcode 26.6 / iOS SDK 26.5
- Host state/callback regressions plus compile/link evidence establish static API and lifecycle contracts only. Simulator execution, camera permission UI, sample delivery, callback timing, and physical-device behavior remain unverified

## B467: construction-time front/back wide-angle camera selection

B467 adds an additive `IosCameraCaptureSession::new_with_camera_position` constructor for `IosCameraPosition::{Front, Back}`. It selects the system default `AVCaptureDeviceTypeBuiltInWideAngleCamera` for `AVMediaTypeVideo` and the requested `AVCaptureDevicePosition` through `defaultDeviceWithDeviceType:mediaType:position:`. It does not enumerate devices. `IosCameraCaptureSession::new()` retains its existing `defaultDeviceWithMediaType:` behavior.

- Apple’s [AVCaptureDevice selector docs](https://developer.apple.com/documentation/avfoundation/avcapturedevice/default%28_%3Afor%3Aposition%3A%29?language=objc) define default-device lookup by type, media, and front/back position, with `nil` when no match exists. The current iPhoneOS SDK header marks both the selector and built-in wide-angle type available from iOS 10.0. Pinned `objc2-av-foundation` 0.3.2 exposes the selector, position constants, and device type through features already enabled by this crate.
- Both constructors keep the existing non-prompting `Authorized` check before input creation. The new constructor returns `CameraCaptureError::NoCameraAtPosition(position)` if AVFoundation returns no matching default device; existing input-creation and session-input errors remain distinct.
- The chosen position is fixed at construction. This does not add camera switching, external/ultra-wide/telephoto/multi-camera enumeration, audio/session mutation, Photos writes, C ABI, or Swift source.
- Host regressions cover successful device pass-through and typed front/back unavailable-device errors. Registered source guards retain the old default-device path, require the new API/availability/error path, and forbid prompt, discovery/enumeration, out-of-scope camera types, audio, Photos, and Swift. Device and Simulator link/import checks retain the selector and wide-angle type reference.
- The profile keeps its existing iOS 10.0 device and iOS 14.0 Simulator link minima. Host tests, compilation, and link/import checks do not prove front/back device availability, preview or media delivery, authorization UI behavior, Simulator execution, or physical-device runtime behavior.
- Final registered command `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-camera-capture-session --format json` returned 102 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED (104 checks) at source head `d889270eae0952aa26cc2c0b6cf9f77ac9b23a08`, snapshot `30e0bed0afeb8c0ffb6af8c8ffbb0258972858400fa1d28b3ff6a47859ca98d9`, on Xcode 26.6 / iOS SDK 26.5. The skipped gates were Simulator execution and physical-device proof; no runtime claim is made.

## B473: selected-camera torch on/off control

B473 adds `IosCameraCaptureSession::set_torch_enabled(bool)` for the device already selected by the session. The implementation uses the public `AVCaptureDevice` torch category and the installed `objc2-av-foundation 0.3.2` bindings; the current iPhoneOS SDK marks the category available from iOS 4.0, below the profile's iOS 10.0 device and iOS 14.0 Simulator floors.

- The configured device remains retained in the serialized capture graph. Torch checks, configuration lock, mode update, and unlock run on the existing private serial session queue.
- `TorchControlError` distinguishes a missing configured device, no torch, unsupported requested mode, unavailable torch when enabling, and configuration-lock failure. Disabling remains allowed when the torch is temporarily unavailable.
- A drop guard unlocks after every successful lock, including an unwind path. No authorization request, device enumeration/switching, session activation/configuration change, audio-session mutation, private selector, or Swift source is added.
- Host regressions cover no-torch, unsupported-mode, unavailable-enable, lock-failure, successful lock/set/unlock order, unavailable disable, and unlock on unwind. The registered link probe requires `hasTorch`, `isTorchAvailable`, `isTorchModeSupported:`, `lockForConfiguration:`, `setTorchMode:`, and `unlockForConfiguration` on both existing targets.
- Host tests and device/Simulator compilation/link checks validate deterministic logic and the public ABI boundary only. Actual torch availability, mode changes, illumination, Simulator execution, and physical-device runtime behavior remain unverified.
