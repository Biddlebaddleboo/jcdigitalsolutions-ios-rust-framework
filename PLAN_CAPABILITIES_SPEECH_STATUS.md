# PLAN_CAPABILITIES_SPEECH_STATUS.md — D53: Speech Authorization State

## Objective

Add one iOS-only Rust read for the app's saved Speech authorization state under capability row `066-ml-vision-language-speech`. This slice adds no portable speech contract

## Public boundary

- `ios_speech_status::authorization_status() -> SpeechAuthorizationStatus` calls only `SFSpeechRecognizer::authorizationStatus`
- Map the four Apple status values to `NotDetermined`, `Denied`, `Restricted`, and `Authorized`. Apple notes that `Denied` may also reflect lack of device support
- Return `Unavailable` below the iOS 10.0 class API floor and `Unknown(isize)` for a future raw enum value
- Treat `Authorized` as saved app authorization only, not service uptime or proof that a recognition task can succeed
- Do not request authorization, create a recognizer, accept audio, create a recognition request, start recognition, or return audio or transcript data
- Keep privacy metadata and any future permission request app-owned

## API evidence

The local iPhoneOS26.5 SDK marks `SFSpeechRecognizer` and `SFSpeechRecognizerAuthorizationStatus` available from iOS 10.0 and unavailable on tvOS. Apple defines `authorizationStatus` as the app's current Speech authorization. The `objc2-speech` 0.3.2 binding exposes `SFSpeechRecognizer::authorizationStatus` and its four enum values behind feature `SFSpeechRecognizer`

The SDK does not declare a main-thread requirement for this class method. Apple’s permission guide requires `NSSpeechRecognitionUsageDescription` for Speech API use; the host app owns this privacy key. This slice does not call the prompt API

## Validation and limits

- Run host, device, and simulator package check, strict Clippy, and rustdoc gates
- Build but do not execute device and simulator link probes; inspect the exact Speech/Foundation import allowlist, Objective-C symbols, and absence of Swift runtime symbols
- Do not add or run tests, request permission, create a recognizer, accept audio, or start recognition
- These gates prove source and link shape only. They do not prove service availability, microphone access, authorization prompt behavior, network availability, or recognition success
