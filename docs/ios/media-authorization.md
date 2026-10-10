# iOS camera and microphone authorization

`ios-media-authorization` implements the portable
`framework_media_authorization::MediaAuthorization` status contract through AVFoundation's
`AVCaptureDevice::authorizationStatusForMediaType:`. `CaptureMedia::Camera` maps only to
`AVMediaTypeVideo`; `CaptureMedia::Microphone` maps only to `AVMediaTypeAudio`. The mapping is
separate for each privacy category. An unrecognized `AVAuthorizationStatus` value becomes
`MediaAuthorizationStatus::Unknown`.

The status call is synchronous and returns a current snapshot. It does not prompt, create
`AVCaptureDeviceInput` or `AVCaptureSession`, enumerate or select hardware, start capture, or access
sample data. The contract cannot report whether hardware exists or is currently available, and
callers must re-query if current authorization state matters.

```rust
use framework_media_authorization::{CaptureMedia, MediaAuthorization};
use ios_media_authorization::IosMediaAuthorization;

let camera = IosMediaAuthorization::authorization_status(CaptureMedia::Camera);
let microphone = IosMediaAuthorization::authorization_status(CaptureMedia::Microphone);
```

An app may explicitly request one permission. The request is a future and AVFoundation work begins
when it is polled, typically by `.await` inside the host app's async context:

```rust,ignore
let mut authorization = IosMediaAuthorization;
let camera = authorization.request_authorization(CaptureMedia::Camera).await;
```

The request invokes `AVCaptureDevice::requestAccessForMediaType_completionHandler` with only the
matching public `AVMediaTypeVideo` or `AVMediaTypeAudio` constant. It returns a fresh
`authorizationStatusForMediaType:` snapshot when Apple invokes its completion block. The callback
may arrive on an arbitrary queue; the future wakes its caller's executor from that callback. No
executor or `Send` requirement is imposed by the portable trait. Dropping an unpolled future starts
no request. Dropping a pending future abandons its result but does not cancel or dismiss native
authorization UI.

Before requesting camera access, the host app must provide `NSCameraUsageDescription`; before
requesting microphone access, it must provide `NSMicrophoneUsageDescription`. Missing purpose text
can terminate the app when access is requested. This crate neither edits nor checks the host app's
property list. No entitlement is added. Authorization requests do not create a capture device,
input, session, audio engine, or output and do not access samples.

## API floor and binding

The installed Xcode 26.6 build 17F113 / iOS SDK 26.5 public `AVCaptureDevice.h` marks
`authorizationStatusForMediaType:` available from iOS 7.0. `AVMediaTypeVideo` and `AVMediaTypeAudio`
are available from iOS 4.0 in `AVMediaFormat.h`; the effective API floor is iOS 7.0. The crate sets
no deployment target. SDK suggested deployment targets are build defaults, not the declared API
floor.

The adapter uses exact `objc2-av-foundation = 0.3.2` with default features disabled and only
`AVCaptureDevice`, `AVMediaFormat`, and `block2`; direct `block2` use enables only its `alloc`
feature. Generated bindings expose the status and request methods, media-type constants, and
`AVAuthorizationStatus`; no local Objective-C shim or Swift source is needed. The safe wrapper
passes only the two documented media-type constants to generated methods whose unsafe bindings
otherwise permit invalid strings that can raise an Objective-C exception.
The package's `AVCaptureDevice` feature necessarily enables the binding crate's `bitflags` and
Foundation collection/value/string features; the binding crate also declares `objc2/std` as a
nonoptional dependency feature. The selected features do not enable AVFAudio, capture sessions,
audio engines, or sample-output APIs.

Apple references: [AVCaptureDevice authorization status](https://developer.apple.com/documentation/avfoundation/avcapturedevice/authorizationstatus%28for%3A%29),
[requesting authorization to capture and save media](https://developer.apple.com/documentation/avfoundation/requesting-authorization-to-capture-and-save-media),
[NSCameraUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nscamerausagedescription),
and [NSMicrophoneUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription).

Host tests exercise deterministic portable request semantics, callback completion/wakeup, and
status mapping. Device and simulator checks establish compile and lint evidence; they do not query
a developer's privacy status, access hardware, display a prompt, or establish capture behavior.
