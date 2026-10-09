# iOS camera and microphone authorization status

`ios-media-authorization` implements the portable
`framework_media_authorization::MediaAuthorization` contract through AVFoundation's
`AVCaptureDevice::authorizationStatusForMediaType:`. `CaptureMedia::Camera` maps only to
`AVMediaTypeVideo`; `CaptureMedia::Microphone` maps only to `AVMediaTypeAudio`. The mapping is
separate for each privacy category. An unrecognized `AVAuthorizationStatus` value becomes
`MediaAuthorizationStatus::Unknown`.

The call is synchronous and returns a current status snapshot. It does not call
`requestAccessForMediaType:completionHandler:`, create `AVCaptureDeviceInput` or
`AVCaptureSession`, enumerate or select hardware, start capture, or access sample data. The
contract cannot report whether hardware exists or is currently available, and callers must re-query
if current authorization state matters.

```rust
use framework_media_authorization::{CaptureMedia, MediaAuthorization};
use ios_media_authorization::IosMediaAuthorization;

let camera = IosMediaAuthorization::authorization_status(CaptureMedia::Camera);
let microphone = IosMediaAuthorization::authorization_status(CaptureMedia::Microphone);
```

For an app that later requests access or attempts camera/microphone capture, Apple requires the host
app's `Info.plist` to include `NSCameraUsageDescription` and/or `NSMicrophoneUsageDescription`, with
purpose text for each used media type. Those keys are not required by this status-only query. This
crate neither edits nor checks the host app's property list. There is no entitlement or permission
prompt in this slice.

## API floor and binding

The installed Xcode 26.6 build 17F113 / iOS SDK 26.5 public `AVCaptureDevice.h` marks
`authorizationStatusForMediaType:` available from iOS 7.0. `AVMediaTypeVideo` and `AVMediaTypeAudio`
are available from iOS 4.0 in `AVMediaFormat.h`; the effective API floor is iOS 7.0. The crate sets
no deployment target. SDK suggested deployment targets are build defaults, not the declared API
floor.

The adapter uses exact `objc2-av-foundation = 0.3.2` with default features disabled and only
`AVCaptureDevice` and `AVMediaFormat`. Generated bindings expose the iOS class method, media-type
constants, and `AVAuthorizationStatus`; no local Objective-C shim or Swift source is needed. The
safe wrapper passes only the two documented media-type constants to the generated method, whose
unsafe binding otherwise permits invalid strings that can raise an Objective-C exception.
The package's `AVCaptureDevice` feature necessarily enables the binding crate's `bitflags` and
Foundation collection/value/string features; the binding crate also declares `objc2/std` as a
nonoptional dependency feature. The selected features do not enable AVFAudio, block2, capture
sessions, or audio engines.

Apple references: [AVCaptureDevice authorization status](https://developer.apple.com/documentation/avfoundation/avcapturedevice/authorizationstatus%28for%3A%29),
[requesting authorization to capture and save media](https://developer.apple.com/documentation/avfoundation/requesting-authorization-to-capture-and-save-media),
[NSCameraUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nscamerausagedescription),
and [NSMicrophoneUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription).

Host tests exercise only deterministic portable values and status mapping. Device and simulator
checks establish compile and lint evidence; they do not query a developer's privacy status, access
hardware, display a prompt, or establish capture behavior.
