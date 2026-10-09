# iOS Speech authorization status

`ios_speech_status::authorization_status()` reads the app's current
`SFSpeechRecognizerAuthorizationStatus` via `SFSpeechRecognizer::authorizationStatus`. It returns
`NotDetermined`, `Denied`, `Restricted`, or `Authorized`; it preserves unrecognized raw values as
`Unknown(isize)` and returns `Unavailable` below iOS 10.0.

This read does not request permission, create an `SFSpeechRecognizer`, accept audio, or start a
recognition task. `Authorized` reports the saved app authorization choice only; it does not prove
service availability or recognition success. Apple notes that `Denied` can also reflect lack of
device support. A host app owns `NSSpeechRecognitionUsageDescription` and any future permission
request.

The inspected SDK marks the class API available from iOS 10.0. Device and Simulator link probes
imported only Speech, Foundation, `libobjc.A.dylib`, and `libSystem.B.dylib`; probes were built and
inspected, not executed. These checks do not prove live authorization values, service availability,
permission prompt behavior, microphone access, or recognition output. The repository's Xcode 27.x
baseline is not met by the local Xcode 26.6 / iOS SDK 26.5 host.

See [D53](../../PLAN_CAPABILITIES_SPEECH_STATUS.md), [B58](../../PLAN_IOS_SPEECH_STATUS.md), and
[G52](../../PLAN_VALIDATION_IOS_SPEECH_STATUS.md) for API evidence and gate results.
