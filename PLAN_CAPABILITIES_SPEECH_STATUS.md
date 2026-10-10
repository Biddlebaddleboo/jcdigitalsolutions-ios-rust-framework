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

## D53 execution evidence — 2026-10-10

- Product source and package docs already meet this boundary; no product edit was needed
- Pinned `ios-rust-build` and `ios-rust-validate` 0.1.0 installed for `x86_64-apple-darwin`; both report source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`
- `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-speech-status` returned `unknown capability`; no validator pilot is registered for this package
- `sh platform/ios/ios-speech-status/scripts/check.sh` passed source guards, `cargo fmt --check`, host `cargo check`, strict host Clippy, host rustdoc, device and simulator checks/Clippy, device rustdoc, zero-Swift-source, shared docs index, and `git diff --check`
- Device and simulator release link probes built and passed `otool`, `nm`, and `strings` inspection; imports matched exactly `Foundation`, `Speech`, `libSystem.B.dylib`, and `libobjc.A.dylib`; `_objc_msgSend` and the Speech class/method names were present; no Swift runtime symbol was found
- Probe executables were not run; no test, permission request, recognizer creation, audio input, recognition request, or recognition task was used
- Evidence does not establish live authorization, service availability, microphone access, prompt behavior, network availability, or recognition success; `Authorized` remains only the app's saved authorization state
