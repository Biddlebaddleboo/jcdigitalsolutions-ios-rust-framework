# iOS Speech authorization status

`ios_speech_status::authorization_status()` reads the app's current `SFSpeechRecognizerAuthorizationStatus` value from `SFSpeechRecognizer::authorizationStatus`. It returns one of the four named Apple states, `Unknown(isize)` for an unrecognized raw value, or `Unavailable` below iOS 10.0

This API does not call `requestAuthorization`, create a recognizer, accept audio, or start a recognition task. `Authorized` means only that the app has a saved Speech authorization choice; it does not prove that the service is available or that recognition can succeed. Apple notes that `Denied` can also reflect lack of device support

Apple marks `SFSpeechRecognizer` as available from iOS 10.0. The local `objc2-speech` 0.3.2 binding exposes `authorizationStatus()` under its `SFSpeechRecognizer` feature. This method has no documented main-thread requirement. Apple’s Speech permission guide requires `NSSpeechRecognitionUsageDescription` for Speech API use; the host app owns this privacy key and any permission request. This method does not call the prompt API

See [D53](../../../PLAN_CAPABILITIES_SPEECH_STATUS.md), [B58](../../../PLAN_IOS_SPEECH_STATUS.md), and the [Apple Speech API](https://developer.apple.com/documentation/speech/sfspeechrecognizer/authorizationstatus%28%29)
