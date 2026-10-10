# PLAN_IOS_MEDIA_AUTHORIZATION.md — Workstream B36: AVFoundation Authorization Status Adapter

## Objective

Implement the D31 camera/microphone status contract through Apple's public AVFoundation authorization query only.

## Dependencies

- D31 `framework-media-authorization`
- `objc2-av-foundation = 0.3.2` with default features disabled and only `AVCaptureDevice`, `AVMediaFormat`, and `block2`; direct `block2 = 0.6.2` enables only `alloc`
- Installed Xcode 26.6 / iOS SDK 26.5 public headers declare both `authorizationStatusForMediaType:` and `requestAccessForMediaType:completionHandler:` at iOS 7.0; the two media constants are available from iOS 4.0

## Write scope

- `PLAN_IOS_MEDIA_AUTHORIZATION.md`
- `platform/ios/ios-media-authorization/**`
- `docs/ios/media-authorization.md`
- The additive request profile in `tools/validation/specs/validation-v1.json` and
  `tools/validation/adapters/check_media_authorization.py`

Root owns workspace and lock integration, canonical capability rows, shared indexes, CI, and aggregate plans. Do not edit those paths.

## Backend behavior

- Export a zero-sized `IosMediaAuthorization` implementation of the static D31 trait
- Map `CaptureMedia::Camera` only to `AVMediaTypeVideo`; map `CaptureMedia::Microphone` only to `AVMediaTypeAudio`
- Call `AVCaptureDevice::authorizationStatusForMediaType` and map NotDetermined/Restricted/Denied/Authorized; map any future raw value to `Unknown`
- Scope unsafe code to the exact generated extern constants and method; pass no caller-provided string
- Keep the status read synchronous and separate from any main-thread/UI contract

## Boundaries

- The `authorization_status` query never calls `requestAccessForMediaType:completionHandler:` or `AVAudioSession.requestRecordPermission`
- Do not create, enumerate, select, configure, or start a device, input, capture session, audio engine, or output
- Do not access camera/microphone samples, request Photos access, add entitlements, or claim runtime privacy behavior
- Host `Info.plist` keys `NSCameraUsageDescription` and `NSMicrophoneUsageDescription` are required before matching requests or capture attempts; the status-only query does not require or install them
- Do not write Swift or Objective-C shim source

## Additive request operation for existing capability rows 046 and 047

B36 remains the completed non-prompting status-query baseline. This selected increment adds an
explicit request path to the existing package and portable contract without allocating or reusing
a numbered B/D workstream ID:

- `IosMediaAuthorization::request_authorization` returns a future and starts
  `AVCaptureDevice::requestAccessForMediaType_completionHandler` only on first poll.
- Camera uses only `AVMediaTypeVideo`; microphone uses only `AVMediaTypeAudio`.
- The callback may run on an arbitrary queue. It re-queries and returns the current AVFoundation
  authorization status, wakes the request future, and catches panics before returning across the
  native callback boundary.
- Dropping an unpolled future starts no request. Dropping a pending future detaches its Rust waker
  and result interest; it does not claim cancellation of AVFoundation work or system UI.
- The matching host `Info.plist` purpose key is required before a request. No host property list is
  generated or edited.
- No device/session enumeration, capture session, audio engine, sample access, or Swift/Objective-C
  shim is in scope.

Acceptance is the portable lazy-request tests, iOS host callback-state tests, device and Simulator
compile/Clippy/rustdoc checks, the source-surface guard, and the registered
`ios-media-authorization` validator profile. These gates do not invoke AVFoundation or display a
permission prompt.

## Checks

- Run host mapper tests, device/simulator target checks, strict Clippy, rustdoc, package format, and the source-surface guard
- Record the exact SDK/build and generated feature set
- Do not run the native authorization query or touch hardware during validation

## Status

B36 is complete in isolated worktree `/Users/john/Projects/.worktrees/jcdig-media-auth-d31` on
`workstream/capabilities-media-auth-d31`, based on `main` HEAD
`7b5513fa2a4864d21a594cbf1fbd43951427155d`. `objc2-av-foundation = 0.3.2` exposes the exact
`authorizationStatusForMediaType:` method and video/audio constants on iOS; enabled features are
only `AVCaptureDevice` and `AVMediaFormat`, with defaults disabled. Xcode 26.6 build 17F113 / iOS
SDK 26.5 headers mark the status method iOS 7.0 and both media constants iOS 4.0. No deployment
target is set.

`./platform/ios/ios-media-authorization/check.sh` passed for B36: two host mapper tests, device and
Simulator checks, host/device/Simulator strict Clippy, device-target rustdoc, and the status-only
source guard. No AVFoundation authorization query, permission prompt, device enumeration, capture,
or sample access ran. Apple usage keys are documented for future request/capture APIs, not for the
status query.

The temporary lock resolution was restored before handoff; root integration added the dependencies
and lock entries required by the additive request operation.

The additive request operation for rows 046/047 is implemented in the current checkout. On Rust
1.94.1, `./crates/framework-media-authorization/check.sh` passed its no-default-feature check,
five unit tests, strict Clippy, and rustdoc. `./platform/ios/ios-media-authorization/check.sh` passed
four host tests, iOS device and Simulator checks, strict Clippy for host and both Apple targets,
device rustdoc, and the updated source-surface guard. The registered
`ios-media-authorization` `ios-rust-validate --capability` profile passed its required Python
adapter, package checks, documentation, zero-Swift-source, and diff gates; the adapter's negative
fixture rejects removal of the AVFoundation request call. No native permission request, prompt,
hardware access, capture, or sample access was executed.
