# PLAN_IOS_SPEECH_STATUS.md — B58: Speech Authorization State Query

## Objective

Expose a narrow iOS Rust read method for the app's current Speech authorization state under row `066-ml-vision-language-speech`

## Scope

The isolated B58 workstream changed `platform/ios/ios-speech-status/**` and this plan. Root
integration adds the central `objc2-speech` workspace dependency, row 066 status, CI gate, shared
validation plan, and iOS documentation index; no portable speech API is added.

## API requirements

- Export only `authorization_status() -> SpeechAuthorizationStatus`
- Call `SFSpeechRecognizer::authorizationStatus` after an iOS 10.0 availability check
- Map the four named Apple states; preserve future values as `Unknown(isize)`
- Do not call `requestAuthorization`, create `SFSpeechRecognizer`, inspect locale, accept audio, create a request, or start recognition
- Do not claim service availability, microphone access, or recognition success
- Keep host privacy metadata and future consent flow outside this package
- Add no main-thread marker because the inspected Apple header does not state a main-thread requirement
- Pin `objc2-speech` 0.3.2 in the workspace dependency table and enable only feature
  `SFSpeechRecognizer`; do not enable `block2`

## Validation and limits

- Run `sh platform/ios/ios-speech-status/scripts/check.sh`
- The link script builds device and simulator probes and inspects imports, Objective-C symbols, and Swift-runtime strings; it never executes a probe
- Do not add or run tests, prompt for permission, capture audio, or start recognition
- These gates do not prove runtime status values, service availability, authorization prompt behavior, network access, or recognition output
