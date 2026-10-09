# PLAN_IOS_REPLAYKIT.md — Workstream B45: ReplayKit Availability Query

## Objective

Implement D40's point-in-time ReplayKit availability snapshot with no capture operation.

## Bounded API

- On iOS 9.0+, call only `RPScreenRecorder.sharedRecorder()` and `isAvailable`
- Return `false` below the declared API floor and on non-iOS targets
- Use `objc2-replay-kit` 0.3.2 with defaults disabled and only `RPScreenRecorder`
- Do not enable `block2`, broadcast features, or `objc2-core-media`
- Do not start/stop recording or capture, broadcast, present a picker, request consent, or alter camera/microphone state
- Preserve Apple's deprecation notice and ScreenCaptureKit recommendation; this workstream does not implement ScreenCaptureKit

## Validation

Run `platform/ios/ios-replaykit/check.sh` for portable and iOS target checks, strict Clippy, rustdoc, source dependencies, and Release import/symbol audit. Link evidence does not establish runtime availability or capture behavior.
