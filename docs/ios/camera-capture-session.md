# iOS camera capture session

`ios-camera-capture-session` exposes an authorized default-camera session, a native preview layer,
processed JPEG capture, video recording, and optional video samples through public AVFoundation.
Session construction does not prompt; request camera access through `ios-media-authorization` and
provide `NSCameraUsageDescription` before an app asks for access.

## Exact built-in camera selection

`IosCameraDeviceSelection` combines `IosCameraDeviceType::{WideAngle, Telephoto}` with
`IosCameraPosition::{Front, Back}`. Use `new_with_camera_selection` at construction or
`set_camera_selection` to switch an existing session. Both types are public AVFoundation device
types available from iOS 10.0; telephoto is found with a one-type, one-position
`AVCaptureDeviceDiscoverySession` query. An unavailable exact selection returns a typed error and
never falls back to another type or position. The existing position-only APIs still select the
default built-in wide-angle camera, and neither API requests permission.

Existing-session changes create the requested device/input before one AVFoundation session
configuration transaction. The old input is restored if the new input is rejected; restoration
failure invalidates the retained camera graph and is reported distinctly. Photo, movie, sample,
preview, and session settings remain attached. Selection is serialized with capture operations and
is rejected during movie finalization or pending shutdown. Static host, device, and Simulator
checks do not prove hardware availability, camera switching, preview continuity, or runtime
behavior.

```rust,ignore
let selection = IosCameraDeviceSelection::new(
    IosCameraDeviceType::Telephoto,
    IosCameraPosition::Back,
);
let session = IosCameraCaptureSession::new_with_camera_selection(main_thread, selection)?;
session.set_camera_selection(IosCameraDeviceSelection::new(
    IosCameraDeviceType::WideAngle,
    IosCameraPosition::Front,
))?;
```

## Live preview orientation

`set_preview_orientation(main_thread, orientation)` sets the live preview layer's public AVFoundation video connection to `Portrait`, `PortraitUpsideDown`, `LandscapeRight`, or `LandscapeLeft`. Call it on the main thread after session creation. The method checks native orientation support before it calls the setter and returns `PreviewOrientationError` if the connection is absent or rejects the choice

Preview orientation is separate from `set_movie_orientation`: it does not change movie files, photos, or video sample buffers. The API uses public `AVCaptureConnection` orientation selectors, deprecated by Apple from iOS 17 but available at the existing iOS 7+ package floor. Static tests and link checks do not prove preview output or device behavior

```rust,ignore
let main_thread = objc2::MainThreadMarker::new().expect("main thread");
session.set_preview_orientation(
    main_thread,
    IosCameraVideoOrientation::Portrait,
)?;
```

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

## Capture-session events

Call `start_observing_events()` to opt in to typed `IosCameraCaptureSessionEvent` values from the owned AVFoundation session, then poll with `poll_event()`. Events cover did-start, did-stop, interruption with an optional `IosCameraSessionInterruptionReason`, interruption-ended, and runtime-error with an optional copied `NSError` domain and code. Unknown interruption codes are preserved. Runtime-error domains are limited to 256 UTF-8 bytes and expose `domain_was_truncated()`.

No event queue or observer is allocated until opt-in; before then, polling returns `None` and the drop count is zero. Observation is idempotent and session-specific. Events posted before observer registration completes are not guaranteed to appear. The 32-item queue drops its oldest event when full; `dropped_event_count()` returns the cumulative saturating drop count. Dropping the session closes event admission, removes observers, and enqueues the existing asynchronous stop. Events do not restart or recover capture. Host tests and target link checks do not prove runtime notification delivery, Simulator behavior, or physical-device behavior.
