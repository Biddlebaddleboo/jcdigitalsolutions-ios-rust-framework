# iOS camera capture session

`ios-camera-capture-session` exposes an authorized default-camera session, a native preview layer,
processed JPEG capture, video recording, and optional native video frames through public AVFoundation
and Core Video.
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

## Native pixel-buffer output

`start_video_frame_output` is an additive polling path alongside the existing borrowed
`CMSampleBuffer` callback. It uses public `AVCaptureVideoDataOutput` and its sample-buffer delegate,
then retains the frame's native `CVPixelBuffer` and copies only the small `CMTime` timestamp value.
Pixel bytes are never copied. Each `IosCameraVideoFrame` owns one retained Core Video buffer; its
`pixel_buffer()` borrow remains valid while the frame value is alive. Use `width()`, `height()`,
`pixel_format()`, and `presentation_time_stamp()` for common metadata, or pass the native pixel
buffer to a compatible image-processing/GPU API. CPU reads must call Core Video's matching
`CVPixelBufferLockBaseAddress` and `CVPixelBufferUnlockBaseAddress` APIs with read-only flags. See
[Apple's `CVPixelBufferLockFlags` docs](https://developer.apple.com/documentation/corevideo/cvpixelbufferlockflags)
and [AVCaptureVideoDataOutput late-frame behavior](https://developer.apple.com/documentation/avfoundation/avcapturevideodataoutput/alwaysdiscardslatevideoframes).

Output setup is queued and does not wait on the main thread. `video_frame_output_status()` reports
`NotStarted`, `Starting`, `Active`, `Failed(error)`, or `Stopped`; a short queue-lock conflict is
reported as `Starting`. `try_next_video_frame()` never waits and returns `None` when the queue is
empty or its producer lock is busy. Start the session first; `Active` means AVFoundation accepted
the output and delegate, not that a device delivered a frame.

`VideoFrameOutputConfiguration::new(capacity)` accepts one through four queued frames, and the
default capacity is two. When the queue is full, the oldest queued frame is released and the new
frame is kept, favoring fresh output. `dropped_video_frame_count()` saturates and includes overflow,
producer-lock contention, pending frames cleared by stop, and late enqueue attempts that reach the
closed queue. AVFoundation callbacks rejected earlier by the inactive-delivery gate do not increment
this count. AVFoundation's `alwaysDiscardsLateVideoFrames` stays enabled on its serial callback
queue, so a slow poll consumer cannot create an unbounded framework queue. The output queue retains
at most its configured number of buffers; frames already returned to the caller are outside that
bound and keep their native buffers alive until dropped. Do not retain returned frames indefinitely.

`stop_video_frame_output`, the existing `stop_video_frame_stream`, session `stop`, and session drop
disable delivery immediately and enqueue queue-clear/output-detach work on the private session
queue. That work releases pending buffers and rejects later delivery without making the caller wait
on the capture callback. A frame already returned from `try_next_video_frame` remains caller-owned.
The existing callback stream and this polling stream are mutually exclusive on one session and each
is terminal after stop. Host queue/configuration tests and device/Simulator compile/link checks do
not prove live camera delivery or pixel contents.

## Capture-session events

Call `start_observing_events()` to opt in to typed `IosCameraCaptureSessionEvent` values from the owned AVFoundation session, then poll with `poll_event()`. Events cover did-start, did-stop, interruption with an optional `IosCameraSessionInterruptionReason`, interruption-ended, and runtime-error with an optional copied `NSError` domain and code. Unknown interruption codes are preserved. Runtime-error domains are limited to 256 UTF-8 bytes and expose `domain_was_truncated()`.

No event queue or observer is allocated until opt-in; before then, polling returns `None` and the drop count is zero. Observation is idempotent and session-specific. Events posted before observer registration completes are not guaranteed to appear. The 32-item queue drops its oldest event when full; `dropped_event_count()` returns the cumulative saturating drop count. Dropping the session closes event admission, removes observers, and enqueues the existing asynchronous stop. Events do not restart or recover capture. Host tests and target link checks do not prove runtime notification delivery, Simulator behavior, or physical-device behavior.
