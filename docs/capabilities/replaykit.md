# ReplayKit availability snapshot

`framework-media::ReplayKitAvailability` stores one allocation-free scalar that reports whether
legacy ReplayKit says its screen recorder is available for recording. The value is transient. It
does not represent consent, permission, a recording session, or a guarantee that a future recording
request will succeed.

Apple currently marks `RPScreenRecorder.isAvailable` deprecated and recommends ScreenCaptureKit's
content-sharing picker availability instead. This slice is deliberately limited to the ReplayKit
row and does not claim current ScreenCaptureKit support or implement capture.

See the [iOS adapter guide](../ios/replaykit.md) for the exact native query, API floor, and runtime
limits.
