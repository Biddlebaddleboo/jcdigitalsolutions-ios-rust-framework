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
- Camera enumeration/selection, explicit switching, torch level control, focus, orientation policy, session notifications, and recovery policy; B473 adds torch on/off and B475 adds an immediate selected-device zoom factor
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

## B475: selected-camera video zoom control

B475 adds `IosCameraCaptureSession::set_zoom_factor(f64)` for the device already retained by the capture graph. The iOS 26.5 `AVCaptureDevice.h` declares `videoZoomFactor`, `activeFormat`, and `AVCaptureDeviceFormat.videoMaxZoomFactor` from iOS 7.0; the existing iOS 10.0 device and iOS 14.0 Simulator floors exceed these APIs. The `minAvailableVideoZoomFactor` and `maxAvailableVideoZoomFactor` properties are iOS 11.0, so the implementation does not use them. Apple documents the setter's valid range as `1.0` through the active format's `videoMaxZoomFactor`; this graph does not enable depth-data delivery.

- The call runs synchronously on the existing session queue, requires the existing selected device, and locks the device before reading its active-format maximum and setting the factor.
- Non-finite and out-of-range factors return typed errors. A valid request is applied exactly; the implementation does not clamp or ramp. `UnlockDeviceConfiguration` releases the lock after success, range failure, invalid native bounds, or unwinding.
- The installed `objc2-av-foundation` 0.3.2 binding exposes `activeFormat`, `videoMaxZoomFactor`, and `setVideoZoomFactor:` with the `objc2-core-foundation` feature. No new crate dependency is added.
- Host regressions cover missing device, non-finite and lower/upper-bound errors, invalid device bounds, lock failure, exact set order, and unlock on range failure/unwind. The registered device and Simulator link probe retains the zoom selectors.
- Static tests and links do not prove live zoom changes, automatic device behavior, Simulator execution, or physical-device behavior.

## B476: selected-camera focus control

B476 adds `IosCameraCaptureSession::set_focus(IosCameraFocusMode, Option<NormalizedFocusPoint>)` for the device already retained by the capture graph. `Auto` autofocuses once, `ContinuousAuto` keeps autofocus active, and `Locked` holds the current lens position. A point is accepted only with an automatic mode because a focus point alone does not start focus and Locked does not apply the supplied point. `None` preserves the current native point

- Coordinates must be finite and in the inclusive `[0.0, 1.0]` range. Apple defines `(0.0, 0.0)` as the image's top-left and `(1.0, 1.0)` as its bottom-right. They are not UIKit preview-layer coordinates; callers must account for preview rotation and mirroring
- Mode and point support are checked before the device configuration lock. After a successful lock, a requested point is set before the focus mode, then `UnlockDeviceConfiguration` releases the lock, including during unwind. Typed errors distinguish a missing selected camera, invalid point, Locked-plus-point, unsupported mode, unsupported point, and lock failure
- Public AVFoundation focus APIs have an iOS 4.0 floor, below the existing iOS 10.0 device and iOS 14.0 Simulator floors. `CGPoint` uses the already-pinned `objc2-core-foundation` binding; no new crate, Swift source, C ABI, permission prompt, enumeration, camera switch, or session/audio policy is added
- Host regressions cover inclusive bounds, NaN/infinity/out-of-range rejection, invalid mode/point combination, unsupported mode/point before lock, missing device, lock failure, point-before-mode order, `None` behavior, and unlock on unwind. Device/Simulator link probes retain the four public mode/point selector references
- Host tests and device/Simulator compile/link checks do not prove autofocus, point orientation, camera hardware support, Simulator execution, or physical-device behavior

## B478: selected-camera exposure metering controls

B478 adds `IosCameraCaptureSession::set_exposure(IosCameraExposureMode, Option<NormalizedFocusPoint>)`, `exposure_target_bias_range()`, and `set_exposure_target_bias(f32)` for the already selected device. The public AVFoundation exposure mode/point APIs have an iOS 4.0 floor and the target-bias APIs have an iOS 8.0 floor, below the existing iOS 10.0 device and iOS 14.0 Simulator deployment floors. The implementation reuses `NormalizedFocusPoint` and its finite inclusive `[0.0, 1.0]` validation; `AutoExpose`, `ContinuousAutoExposure`, and `Locked` map directly to the corresponding native modes. `Custom` is intentionally omitted because complete manual exposure requires duration/ISO controls outside this increment.

- Exposure mode and optional point support are checked on the selected device before locking. `Locked` with a point is rejected; `None` leaves the native point unchanged. An accepted point is set before the exact requested mode under the existing configuration lock/drop guard.
- Bias-range reads return the selected device's native finite ordered inclusive bounds. Bias writes reject non-finite, invalid-range, and out-of-range values without clamping; they read bounds while locked and pass the exact accepted value to `setExposureTargetBias:completionHandler:` with a nil completion handler. Success means AVFoundation accepted the request, not that a captured frame applied it.
- Host regressions cover missing-device short-circuit, each invalid coordinate dimension/value class, Locked-plus-point, unsupported mode/point, point-before-mode and `None` behavior, lock failure and unwind, invalid native bias ranges, non-finite/out-of-range bias, exact inclusive endpoints, native call order, and invalid range snapshots. The registered link gate retains only selectors exercised by this setter/query API: mode/point support and setters, native minimum/maximum bias, and target-bias setter; unused `exposureMode` and `exposurePointOfInterest` property getters are not asserted.
- The registered `ios-camera-capture-session` profile reports 178 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED. Host logic, static source guards, and iOS device/Simulator compilation/link-selector checks do not prove metering behavior, exposure-point mapping through preview orientation/transforms, Simulator execution, or physical-device runtime behavior. Custom/manual ISO and duration, completion callbacks, preview-coordinate conversion, format/session changes, permission prompts, C ABI, Swift source, and audio remain outside B478.

## B479: opt-in microphone audio in camera movie files

B479 adds `IosCameraCaptureSession::start_video_recording_with_audio` as a per-movie opt-in beside the unchanged, video-only `start_video_recording`. The existing `AVCaptureMovieFileOutput` records the video and public default microphone input in one QuickTime movie; the audio input is absent during preview/idle and is removed on the private serial capture queue at terminal settlement.

- The new path checks current microphone authorization through `IosMediaAuthorization::authorization_status(CaptureMedia::Microphone)` before it calls the AVFoundation input factory. It never requests permission. Non-Authorized states map to `MicrophoneAuthorizationNotGranted`; missing media type/device, input-creation failure, session rejection, and a stale attached input have distinct `VideoRecordingError` variants.
- Audio input attach uses public `AVMediaTypeAudio`, `AVCaptureDevice.defaultDeviceWithMediaType:`, `AVCaptureDeviceInput.deviceInputWithDevice:error:`, `canAddInput:`, and session configuration calls. It sets only `AVCaptureSession.automaticallyConfiguresApplicationAudioSession(false)`; it does not access or mutate the host's audio-session category, mode, active state, or route. The host must configure and activate an input-capable session before this call and supply `NSMicrophoneUsageDescription` for the app.
- Normal terminal callback, native failure, callback-unwind fallback, and setup-failure cleanup all remove any attached audio input before completion or deferred session stop. The registered host regression checks that unauthorized statuses cannot reach the input factory, the legacy video-only method does not prepare audio, and terminal cleanup precedes completion/deferred stop.
- The registered `ios-camera-capture-session` profile requires the new selectors and the existing iOS 10.0 device/iOS 14.0 Simulator deployment floors. It preserves host tests, source/negative guards, device and Simulator compilation/Clippy, Rustdoc, and link/import gates. A static link or host state test does not prove microphone authorization UI, audio-session compatibility, captured audio, movie playback, Simulator execution, or physical-device runtime behavior.
- B479 does not extend B455 or B478's historical scopes. It adds no audio sample callback, audio file/data output, direct audio-session API, permission prompt, entitlement, Photos write, dependency, C ABI, private API, or Swift source.

## B482: caller-selected movie-file orientation

B482 adds `IosCameraCaptureSession::set_movie_orientation(IosCameraVideoOrientation)` for the existing `AVCaptureMovieFileOutput` video connection. The caller selects `Portrait`, `PortraitUpsideDown`, `LandscapeRight`, or `LandscapeLeft`; the choice applies to later video-only or B479 audio-enabled movie requests. AVFoundation's default remains unchanged until a call succeeds.

- The iPhoneOS 26.5 SDK marks `AVCaptureVideoOrientation`, `isVideoOrientationSupported`, and `videoOrientation` deprecated from iOS 17.0 while retaining their public iOS 4.0 availability; `connectionWithMediaType:` is public from iOS 5.0. Both floors are below the package's iOS 10.0 device and iOS 14.0 Simulator minima, so this is the smallest compatible public path. A narrow `#[allow(deprecated)]` covers only the native support check and setter. The iOS 17 `videoRotationAngle` API cannot cover either current floor and does not provide a direct device-independent substitute for the caller-selected AVFoundation orientation enum.
- The method runs synchronously on the private serial capture queue. It rejects pending shutdown, an active movie through terminal finalization, a missing output/connection, and an unsupported orientation; it checks `isVideoOrientationSupported` before `setVideoOrientation:`. Queue order ensures a call enqueued after movie start cannot change that active file; an earlier successful call selects the next movie orientation.
- The operation changes only the movie-file connection. It does not rotate the preview layer, still photos, or video sample stream, inspect device/UI orientation, add `UIDeviceOrientation`, CoreMotion, or a UI dependency, or alter B479 audio input/session lifecycle.
- Host tests cover exact enum mapping, unsupported-before-set, unchanged prior orientation on failure, absent output/connection, active/finalizing state, and pending shutdown. Registered device/Simulator link selectors cover connection lookup, support query, and setter; all existing camera profile gates and the iOS 10.0 / iOS 14.0 floors remain required.
- Host tests and static compile/link checks do not prove the encoded movie's displayed orientation, playback, Simulator execution, or physical-device behavior.
- The final registered `ios-camera-capture-session` profile passed on snapshot `383e75a03106b5667f61ff35858e8b406c4b76a3680510fd5f3349ffb2852e55`: 222 PASS, 0 FAIL, 0 BLOCKED, 2 SKIPPED. The skips were Simulator execution (`validation builds and inspects link probes only; no simulator execution is claimed`) and physical-device proof (`no attached-device runtime test is part of this validation pilot`).

## B486: runtime camera position switch

B486 adds `IosCameraCaptureSession::set_camera_position(IosCameraPosition)` to replace the existing session's selected video input with the public default built-in wide-angle front or back device. It preserves `new()` behavior and session outputs/settings; no enumeration, permission prompt, or app audio-session change is added.

- The synchronous request runs on the existing serial capture queue, returns without mutation for the already selected position, and rejects missing camera state, active/finalizing movie recording, and deferred shutdown. Device and input creation precede session mutation.
- One `beginConfiguration` / `commitConfiguration` transaction removes the old input and adds the accepted replacement. If AVFoundation rejects the candidate, the old input is restored in the same transaction; failure to restore returns a distinct error and clears stale retained graph state.
- Host regressions cover no-op and busy preflight, missing requested device, queue dispatch/reentrancy, replacement order, rejected-input rollback, rollback failure, and graph-state commit. Registered link probes retain the public position selector and queue identity symbol for iOS device and Simulator targets.
- The final registered `ios-camera-capture-session` profile passed 246 checks, with 0 FAIL, 0 BLOCKED, and 2 SKIPPED, on source head `1f2234ac990fe07669df958aef43f04e8388a672` and snapshot `daf896766941146f6112303720edde02f946aa64d2277efc4d9abb79c073fca4` using Xcode 26.6 / iOS SDK 26.5. Host tests, format, Clippy, Rustdoc, device and Simulator compilation, and link/import checks passed. The skips were Simulator execution and physical-device proof; camera switching and preview continuity remain runtime-unverified.

## B490: selected-camera white-balance modes

B490 adds `IosCameraWhiteBalanceMode::{Locked, AutoWhiteBalance, ContinuousAutoWhiteBalance}`, `IosCameraCaptureSession::is_white_balance_mode_supported`, and `IosCameraCaptureSession::set_white_balance_mode` for the device already selected by the session. `Locked` holds the current white balance, `AutoWhiteBalance` adjusts once and then locks, and `ContinuousAutoWhiteBalance` continues automatic adjustment.

- The iPhoneOS 26.5 SDK exposes the public white-balance category, support query, and mode property from iOS 4.0. The installed `objc2-av-foundation` 0.3.2 bindings provide all three typed mode values and both selectors through the existing `AVCaptureDevice` feature; the iOS 10.0 device and iOS 14.0 Simulator floors remain unchanged.
- Support queries and mode changes run on the existing serial capture queue and use only the retained selected device. The setter checks support before locking, reports typed missing-device, unsupported-mode, and configuration-lock errors, and releases a successful lock on return or unwind.
- Host regressions cover missing device, unsupported-before-lock, lock failure, support/set/unlock order, and unlock after unwind. The registered link probe retains both public selectors on device and Simulator targets.
- The final registered `ios-camera-capture-session` profile passed 267 checks, with 0 FAIL, 0 BLOCKED, and 2 SKIPPED, at source head `5731f8a2e7a8aafe68731d0791638599c326260e` and snapshot `20903af39904eb5b439667382f6bf4d9b72f9c407133c9529f14913f1ce3276b` using Xcode 26.6 / iOS SDK 26.5. Host tests, format, Clippy, Rustdoc, iOS 10.0 device and iOS 14.0 Simulator compilation, device Rustdoc, source guards, documentation, policy, and link/import checks passed. Simulator execution and physical-device proof were skipped; white-balance response remains runtime-unverified.
- This adds no device enumeration/switching, permission prompt, session/audio-policy mutation, manual gains, temperature/tint, C ABI, dependency, or Swift source.

## B495: selected-camera exact custom exposure

B495 closes the custom/manual ISO and duration implementation gap that B478 recorded outside its historical scope. It adds `IosCameraCaptureSession::custom_exposure_range()` and `set_custom_exposure(MediaTime, f32)` for the session's existing selected device; it does not add `Custom` to `IosCameraExposureMode`.

- The range snapshot exposes inclusive active-format `minExposureDuration`, `maxExposureDuration`, `minISO`, and `maxISO`. Native durations become `framework_media::MediaTime` only when `CMTime` is valid, finite, positive, and has epoch zero; the original integer value and positive timescale are preserved exactly. ISO values retain AVFoundation's native `f32` representation.
- A setter request rejects a non-positive rational duration or non-finite/non-positive ISO before native calls. It checks `AVCaptureExposureModeCustom` support before `lockForConfiguration`, reads and validates the active format's current bounds under the successful lock, rejects out-of-range values without clamp, then calls public `setExposureModeCustomWithDuration:ISO:completionHandler:` with nil completion. The existing drop guard unlocks on success, validation failure, or unwind.
- A range query is only a point-in-time snapshot; a write always reads the current active format again. Success means AVFoundation accepted the request, not that a particular frame or processed photo used the duration/ISO. Still-photo quality prioritization is unchanged.
- Host regressions cover exact rational comparison and inclusive endpoints, invalid native duration flags/epoch/timescale/values, invalid ISO bounds, invalid request values, missing selected camera, unsupported-before-lock, no-clamp range failures, lock failure, native call order, and unlock on unwind. The registered link probe retains active-format bound getters and the custom-exposure selector for both existing target floors.
- B495 adds no permission prompt, camera enumeration/switching, `Custom` automatic-mode enum variant, completion callback, session/audio policy change, external dependency, C ABI, private symbol, or Swift source. Device/Simulator compilation and selector links do not prove live exposure response, a captured frame's settings, Simulator execution, or physical-device behavior.

## B545: per-photo flash mode

B545 adds `IosCameraCaptureSession::capture_photo_with_flash(IosCameraFlashMode, completion)` with `Off`, `On`, and `Auto` requests. The method checks the configured `AVCapturePhotoOutput::supportedFlashModes` before creating a request and returns `PhotoCaptureError::FlashModeUnsupported` without submitting a capture when the mode is absent. It applies `setFlashMode:` only to the fresh settings for that request. Existing `capture_photo` does not query or override flash settings.

- Host regressions cover all advertised modes, unsupported `On`/`Auto` rejection before capture, and preservation of the existing default-settings path. The device/Simulator link probe exercises the new API; the link gate retains `supportedFlashModes`, `containsObject:`, `numberWithInteger:`, and `setFlashMode:` at the existing iOS 10.0 device / iOS 14.0 Simulator floors.
- The final registered `ios-camera-capture-session` profile exited 0 with 265 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED at source snapshot `4a665e1f9ab882455b3717d44ea6850499a03f97a6be6ce8a4000ca143666506`. Simulator execution and physical-device proof were skipped; the profile proves compilation, linkage, and static behavior only, not flash firing or camera runtime behavior.
- B545 adds no default-setting change, permission prompt, session policy, device selection, Photos write, C ABI, private API, or Swift source.

## B560: caller-selected capture-session quality preset

B560 adds `IosCameraSessionPreset` and `IosCameraCaptureSession::set_session_preset`. Typed choices map to public AVFoundation photo, quality, and resolution presets from the existing high-quality default through UHD 3840×2160. The default remains unchanged until a request succeeds. Each request runs on the existing serial session queue, rejects active/finalizing movie capture and deferred movie-driven shutdown, checks `canSetSessionPreset:` before one `beginConfiguration`/`setSessionPreset:`/`commitConfiguration` transaction, and returns `PresetUnsupported` without mutation when the current graph declines the choice. AVFoundation allows preset changes while running; requested dimensions do not guarantee delivered frame dimensions. No device format is directly selected.

Host regressions cover every typed-to-native mapping, support-query and transaction order, unsupported-preset no-mutation, and active/finalizing/shutdown rejection before support query. The registered device and Simulator link probe retains every preset constant and both session-preset selectors. Static host/device/Simulator checks do not prove capture output dimensions, preview changes, runtime capture, Simulator execution, or physical-device behavior.

- The registered `ios-camera-capture-session` profile passed 266 checks, with 0 FAIL, 0 BLOCKED, and 2 SKIPPED at B560 snapshot `0a7eff784ddba1d537d4c1aa4c21f122efb22c2e3e553be9645b852fef708ccc`. The skipped checks were Simulator execution and physical-device proof.

## B564: per-photo quality prioritization

B564 adds `IosCameraPhotoQuality::{Speed, Balanced, Quality}` and `capture_photo_with_quality` to select AVFoundation's public per-photo quality-versus-delivery-speed preference. Existing `capture_photo` and `capture_photo_with_flash` continue to use their current settings. An opt-in `set_max_photo_quality_prioritization` raises the output ceiling before session start; AVFoundation's default Balanced ceiling remains unchanged unless the caller requests otherwise. A Speed ceiling is rejected with a typed error so it cannot invalidate legacy default Balanced settings.

- Quality requests are checked against the output's current `maxPhotoQualityPrioritization` before request submission and fail with `PhotoCaptureError::PhotoQualityPrioritizationUnsupported` when the request exceeds it. Speed and Balanced remain available under the default ceiling; Quality requires an explicit ceiling change.
- The ceiling setter requires iOS 13, a configured photo output, a stopped session, no active/finalizing movie, and no pending shutdown. The stopped-only policy avoids AVFoundation's documented potentially lengthy running-session rebuild and preview disruption. AVFoundation may also change the ceiling after camera/format changes, so each photo request checks the current value.
- Host regressions cover default-setting preservation, allowed/rejected priorities, and setter rejection for Speed, running, recording, missing-output, and shutdown states; rejected setters do not mutate the output. The existing device/Simulator link probe retains the public output getter/setter and per-photo setting selectors; deployment minima remain iOS 10.0 device and iOS 14.0 Simulator.
- Host, compile, and link checks establish source and ABI contracts only. They do not prove image quality, processing duration, camera/preview behavior, Simulator execution, or physical-device runtime.
- Registered command `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-camera-capture-session --diagnostics short` passed 269 checks, with 0 FAIL, 0 BLOCKED, and 2 SKIPPED at snapshot `4d160a103788a68c3c9f1a7aba5610a59dbd672514ed9ddda9701964f6c55c97`. The skipped checks were Simulator execution and physical-device proof.

## B572: caller-selected live-preview orientation

B572 adds `IosCameraCaptureSession::set_preview_orientation(MainThreadMarker, IosCameraVideoOrientation)` for the existing preview layer. The method uses the public preview-layer connection, checks `isVideoOrientationSupported` before `setVideoOrientation:`, and returns a typed error when the connection is absent or rejects the requested orientation. It is distinct from B482's movie-file orientation and does not alter photos or sample output

- The native preview layer stays main-thread-affine. This increment adds no UI framework dependency, app-orientation inference, mirroring policy, capture session mutation, permission request, C ABI, private API, or Swift source
- Host regressions cover missing-connection short-circuit, unsupported-before-set with preserved prior value, and exact supported setter value. The target link probe covers the `connection`, support, and orientation selectors on iOS device and Simulator floors
- Static tests and target link checks do not prove preview orientation, camera use, Simulator execution, or physical-device behavior
- Registered command `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-camera-capture-session --diagnostics short` returned 267 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED on snapshot `a6ae3b7e5b70f130f337e221b9bcfddbba904f92f933878972326fc04204361d`. The skips were Simulator execution and physical-device proof

## B588: typed built-in camera-device selection

B588 adds `IosCameraDeviceType::{WideAngle, Telephoto}` and `IosCameraDeviceSelection`, pairing either public built-in video type with `IosCameraPosition::{Front, Back}`. `new_with_camera_selection` constructs a session for the exact selection; `set_camera_selection` switches the existing session. The existing `new()`, `new_with_camera_position`, and `set_camera_position` contracts remain intact. The position-only switch still restores wide-angle if an explicitly selected telephoto device is active at the same position.

- Both public device-type constants and `AVCaptureDeviceDiscoverySession` are available from the current iOS 10.0 device floor. A one-type, one-position video query is used because AVFoundation requires discovery for telephoto devices. The returned device type and position are checked again; no fallback, prompt, or all-device scan is performed. Later built-in types with higher floors are excluded because the current `objc2-av-foundation` bindings expose their globals without weak-link availability metadata.
- New-device and input creation precede existing-session mutation. A successful switch uses one `beginConfiguration` / `commitConfiguration` transaction and retains the prior input when AVFoundation rejects the candidate; failed restoration reports `InputRollbackFailed` and clears retained device, input, and selection state. Photo/movie/sample outputs, preview, and session settings are unchanged. Active/finalizing movies and pending shutdown reject before discovery.
- Host regressions cover exact type/position matching, unavailable exact selections, same-selection no-op, type-only and position changes, busy/missing states, rollback, stale-state clearing, and commit-after-success. The registered profile link probe invokes both type variants and retains discovery/type/property symbols on device and Simulator targets.
- Selection availability, live switching, output continuity, preview behavior, Simulator execution, and physical-device behavior remain runtime-unverified. The existing validator's runtime-only skips are not represented as passes.
- Registered `ios-camera-capture-session` validation passed 270 checks with 0 FAIL, 0 BLOCKED, and 2 SKIPPED at snapshot `a3f5c02de0f12c04d5a749030e8acc12fd04ff429d34d8390b1b27ff83448d78`. The skips are Simulator execution and physical-device proof; target compilation, strict Clippy, Rustdoc, and release link/import checks are static evidence only.
