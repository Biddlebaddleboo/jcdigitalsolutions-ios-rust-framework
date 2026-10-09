# Camera and microphone authorization status

`framework-media-authorization` defines a `#![no_std]` status-only contract for camera and
microphone permission state. `CaptureMedia` keeps the two authorization domains distinct, and
`MediaAuthorizationStatus` preserves `NotDetermined`, `Restricted`, `Denied`, `Authorized`, and
`Unknown` without exposing an Apple type.

`MediaAuthorization::authorization_status` returns a snapshot. `Authorized` means only that the
selected platform reports authorization; it does not guarantee a connected device, an available
input, or a usable capture session. Re-query when current status matters because the user or
operating system can change it outside the process. `Unknown` safely represents an unrecognized
platform status.

This contract does not request permission, access camera or microphone samples, enumerate devices,
create a session, configure audio, capture, or record. It is a bounded authorization slice, not a
portable camera or audio API. Camera and microphone authorization are separate from Photos library
authorization.

For future permission requests or capture operations, the host app must provide the exact iOS
purpose keys `NSCameraUsageDescription` and `NSMicrophoneUsageDescription` for the media it uses.
This status-only contract does not request permission or attempt capture and does not itself require
either purpose key.
