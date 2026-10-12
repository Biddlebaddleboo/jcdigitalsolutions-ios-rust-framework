# iOS camera capture session

`ios-camera-capture-session` exposes an authorized default-camera session, a native preview layer,
processed JPEG capture, video recording, and optional video samples through public AVFoundation.
Session construction does not prompt; request camera access through `ios-media-authorization` and
provide `NSCameraUsageDescription` before an app asks for access.

## Per-photo quality

`IosCameraPhotoQuality::{Speed, Balanced, Quality}` selects AVFoundation's preference for image
quality versus delivery speed on one processed JPEG request. `capture_photo_with_quality` requires
iOS 13 or later. `Speed` favors prompt delivery, `Balanced` trades quality against speed, and
`Quality` may take substantially longer.

For `AVCaptureSession`, AVFoundation's output ceiling defaults to `Balanced`. The existing
`capture_photo` and `capture_photo_with_flash` methods keep their default settings. They do not read
or change the ceiling. A `Speed` ceiling is rejected with
`PhotoQualityPrioritizationError::MaximumBelowDefault` so the ceiling cannot invalidate those
legacy default settings. Set the `Quality` ceiling before session start only when a request needs
it:

```rust,ignore
session.set_max_photo_quality_prioritization(IosCameraPhotoQuality::Quality)?;
session.capture_photo_with_quality(IosCameraPhotoQuality::Quality, |result| {
    // `result` owns the processed JPEG bytes or a typed capture error
})?;
```

The ceiling method is iOS 13+ and rejects a running session, an active/finalizing movie, pending
shutdown, or a missing photo output. AVFoundation may rebuild a running session and disrupt preview
when this ceiling changes, so this API allows the change only while stopped. A quality request
above the current ceiling calls back with `PhotoCaptureError::PhotoQualityPrioritizationUnsupported`
before AVFoundation receives the request. Camera or format changes may alter the native ceiling;
each request reads the current value. The default methods remain unchanged.

Capture callbacks are not guaranteed on the main thread. Keep the session alive until its callback
runs and hop to the main thread before UIKit work. Host tests, iOS device/Simulator compilation, and
link checks verify deterministic state rules and public API linkage only; they do not prove image
quality, processing time, preview behavior, capture, Simulator execution, or physical-device
runtime.
