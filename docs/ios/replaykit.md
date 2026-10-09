# iOS ReplayKit availability snapshot

`ios-replaykit::availability_snapshot()` calls only `RPScreenRecorder.sharedRecorder()` and reads
`isAvailable`. It starts no capture, recording, broadcast, picker, microphone, or camera operation.
The scalar may change immediately; a positive result is not consent or a promise that a later
recording request will succeed. The API returns `false` below its iOS 9.0 floor and on non-iOS
targets.

Apple's current API documentation marks `isAvailable` deprecated and recommends
`SCContentSharingPicker.isAvailable`. This adapter stays scoped to legacy ReplayKit; ScreenCaptureKit
capture, picker presentation, recording, and permission flows remain outside scope.

The binding is `objc2-replay-kit` 0.3.2 with default features disabled and only `RPScreenRecorder`
enabled. It does not enable `block2`, broadcast features, or `objc2-core-media`.

Run `sh platform/ios/ios-replaykit/check.sh` for portable and iOS device/Simulator checks, strict
Clippy, rustdoc, exact Release imports/symbols, docs, and zero-Swift-source checks. Probes are linked,
not run; no live availability or capture behavior is claimed.

## References

- [Apple `RPScreenRecorder`](https://developer.apple.com/documentation/replaykit/rpscreenrecorder)
- [Apple `isAvailable`](https://developer.apple.com/documentation/replaykit/rpscreenrecorder/isavailable?language=objc)
- [Apple `startRecordingWithHandler:` prompt behavior](https://developer.apple.com/documentation/replaykit/rpscreenrecorder/startrecording%28handler%3A%29?language=objc)
- [`objc2-replay-kit` 0.3.2 binding](https://docs.rs/objc2-replay-kit/0.3.2/objc2_replay_kit/struct.RPScreenRecorder.html)
