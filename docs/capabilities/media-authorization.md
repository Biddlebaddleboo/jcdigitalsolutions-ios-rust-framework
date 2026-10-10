# Camera and microphone authorization

`framework-media-authorization` defines `#![no_std]` contracts for camera and microphone
authorization. `CaptureMedia` keeps the two authorization domains distinct, and
`MediaAuthorizationStatus` preserves `NotDetermined`, `Restricted`, `Denied`, `Authorized`, and
`Unknown` without exposing an Apple type.

`MediaAuthorization::authorization_status` returns a snapshot. `Authorized` means only that the
selected platform reports authorization; it does not guarantee a connected device, an available
input, or a usable capture session. Re-query when current status matters because the user or
operating system can change it outside the process. `Unknown` safely represents an unrecognized
platform status.

`MediaAuthorization::authorization_status` is non-prompting. `MediaAuthorizationRequester` exposes
an explicit request for one selected media kind as a future. Backends must defer prompt-capable
native work until the future's first poll. Dropping an unpolled future starts no request; dropping a
pending future abandons its Rust result but does not promise to cancel a prompt or native request.
The request returns authorization status only. Neither contract accesses camera or microphone
samples, enumerates devices, creates a session, configures audio, captures, or records. Camera and
microphone authorization are separate from Photos library authorization.

Before an iOS request, the host app must provide the matching purpose key: `NSCameraUsageDescription`
for camera access and `NSMicrophoneUsageDescription` for microphone access. This library does not
edit or validate the host app's property list. A status query does not require either key.
