# PLAN_IOS_MEDIA_AUTHORIZATION.md — Workstream B36: AVFoundation Authorization Status Adapter

## Objective

Implement the D31 camera/microphone status contract through Apple's public AVFoundation authorization query only.

## Dependencies

- D31 `framework-media-authorization`
- `objc2-av-foundation = 0.3.2` with default features disabled and only `AVCaptureDevice` and `AVMediaFormat`
- Installed Xcode 26.6 / iOS SDK 26.5 public headers declare `authorizationStatusForMediaType:` at iOS 7.0 and the two media constants at iOS 4.0

## Write scope

- `PLAN_IOS_MEDIA_AUTHORIZATION.md`
- `platform/ios/ios-media-authorization/**`
- `docs/ios/media-authorization.md`

Root owns workspace and lock integration, canonical capability rows, shared indexes, CI, and aggregate plans. Do not edit those paths.

## Backend behavior

- Export a zero-sized `IosMediaAuthorization` implementation of the static D31 trait
- Map `CaptureMedia::Camera` only to `AVMediaTypeVideo`; map `CaptureMedia::Microphone` only to `AVMediaTypeAudio`
- Call `AVCaptureDevice::authorizationStatusForMediaType` and map NotDetermined/Restricted/Denied/Authorized; map any future raw value to `Unknown`
- Scope unsafe code to the exact generated extern constants and method; pass no caller-provided string
- Keep the status read synchronous and separate from any main-thread/UI contract

## Boundaries

- Never call `requestAccessForMediaType:completionHandler:` or `AVAudioSession.requestRecordPermission`
- Do not create, enumerate, select, configure, or start a device, input, capture session, audio engine, or output
- Do not access camera/microphone samples, request Photos access, add entitlements, or claim runtime privacy behavior
- Host `Info.plist` keys `NSCameraUsageDescription` and `NSMicrophoneUsageDescription` are required before future requests or capture attempts; this status-only call does not require or install them
- Do not write Swift or Objective-C shim source

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

`./platform/ios/ios-media-authorization/check.sh` passes: two host mapper tests, device and simulator
checks, host/device/simulator strict Clippy, device-target rustdoc, and the status-only source guard.
No AVFoundation authorization query, permission prompt, device enumeration, capture, or sample
access ran. Apple usage keys are documented for future request/capture APIs, not for this query.

The temporary lock resolution was restored before handoff; root must add the exact dependency and
workspace lock entries during integration.
