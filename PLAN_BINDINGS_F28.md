# PLAN_BINDINGS_F28.md — F28: Speech authorization-status C ABI

## Objective

Expose only B58's existing iOS Speech authorization-state query through an opt-in C function. Do
not add a portable speech API, permission request, recognizer, audio input, recognition task, or
service-readiness claim

## Candidate

B58/D53 is the selected partial backend. `platform/ios/ios-speech-status` exposes only
`authorization_status() -> SpeechAuthorizationStatus`; its typed `objc2-speech` 0.3.2 binding uses
`SFSpeechRecognizer::authorizationStatus`, checks iOS 10.0 availability first, and preserves
unknown `NSInteger` values. The local iPhoneOS 26.5 SDK declares the enum as `NSInteger`, assigns
the four named values 0 through 3, and marks `SFSpeechRecognizer` available from iOS 10.0. Apple's
class-method docs describe a read of the app's current authorization; the separate
`requestAuthorization` method asks the user and can prompt. F28 calls only the getter

Other existing partials were not selected: B53's SafetyKit getter has an unresolved
getter-specific entitlement prerequisite; B55's Game Center result reads signed local-player
state; B60's StoreKit 1 purchase-ability API is deprecated; B61 RoomPlan and B63 StoreKit 2 need
Swift ABI thunks. These do not block the bounded B58 contract

Remaining `X` rows stay out of scope. FamilyControls lacks a supported Rust route and documented
query-only entitlement semantics; Foundation Models, ContactProvider, ExtensionFoundation,
MarketplaceKit, MatterSupport, and several service APIs are Swift-only. DeviceActivity getter
meaning and entitlement semantics remain unclear. PushToTalk, CarPlay, browser/extension, and
locked-camera flows need entitlement, APNs, host lifecycle, UI, or session state. HomeKit's first
manager use can prompt. See each capability feasibility plan; F28 adds no capability row and does
not change matrix counts

## Exact ABI

- Cargo feature: `ios-speech-status`; default remains empty
- Header: `bindings/c/include/framework_ios_speech_status.h`
- Export: `FrameworkStatus framework_ios_speech_status_authorization_status(FrameworkIosSpeechAuthorizationStatus *out_raw_status)`
- `FrameworkIosSpeechAuthorizationStatus` is signed `int64_t`; named constants mirror the native
  values `NOT_DETERMINED` 0, `DENIED` 1, `RESTRICTED` 2, `AUTHORIZED` 3. Unknown signed values
  pass through unchanged
- A non-null output must address valid, aligned writable `int64_t` memory during the synchronous
  call. Nullness is the only pointer check; the pointer is not retained and the caller must avoid
  unsynchronized concurrent access
- iOS 10.0 and later: `FRAMEWORK_STATUS_OK` with the raw native authorization code
- iOS below 10.0: `FRAMEWORK_STATUS_UNAVAILABLE` and zero output, per B58's runtime guard
- Non-iOS: `FRAMEWORK_STATUS_UNSUPPORTED` and zero output; the iOS backend is absent from that
  target feature graph
- Null output: `FRAMEWORK_STATUS_INVALID_ARGUMENT` with no write
- Caught panic: `FRAMEWORK_STATUS_PANIC`; output remains zero and no unwind crosses C
- Calls run synchronously on the caller's thread. Apple documents no main-thread rule for the
  class method; F28 adds no broader thread-safety promise
- No Objective-C object, recognizer, audio data, callback, or native pointer crosses C

The status code is separate from the output value: raw zero is a valid `NOT_DETERMINED` status,
so callers must inspect `FrameworkStatus` before reading the output. `NSSpeechRecognitionUsageDescription`
requirements for the app's permission/recognition use are not changed by this status-only wrapper

## Source evidence

- Apple, [`SFSpeechRecognizer.authorizationStatus()`](https://developer.apple.com/documentation/speech/sfspeechrecognizer/authorizationstatus%28%29): current app authorization status
- Apple, [`SFSpeechRecognizerAuthorizationStatus`](https://developer.apple.com/documentation/speech/sfspeechrecognizerauthorizationstatus): named status values
- Installed SDK: `$(xcrun --sdk iphoneos --show-sdk-path)/System/Library/Frameworks/Speech.framework/Headers/SFSpeechRecognizer.h`, with iOS 10.0 class availability and the `authorizationStatus` declaration
- Installed Rust binding: `objc2-speech` 0.3.2 generated `SFSpeechRecognizerAuthorizationStatus(pub NSInteger)` and `authorizationStatus() -> SFSpeechRecognizerAuthorizationStatus`; no `block2` feature is needed

## Validation and evidence

`cargo +1.94.1 check --offline -p framework-c-api --no-default-features --features
ios-speech-status` passed in the integrated checkout.
`sh bindings/c/check-ios-speech-status.sh` passed: the feature graph stays opt-in and target-iOS,
host/device/Simulator checks, strict Clippy, rustdoc, ABI assertions, and C11/C++17 header syntax
all passed. The static gate asserts source, header, guide, and plan output-pointer conditions:
valid aligned writable `int64_t` storage for the full synchronous call, caller protection from
unsynchronized access, zero initialization before platform handling, nullness-only validation, and
no pointer retention. It does not prove arbitrary C memory validity.
`sh bindings/c/check-ios-speech-status-link.sh` passed Release C11/C++17 host, device,
and Simulator links, export parity, forbidden-API checks, exact import allowlists, and deployment
metadata. Host C and C++ import only `libSystem.B.dylib`. Device and Simulator C and C++ imports
are exactly Foundation, Speech, `libSystem.B.dylib`, and `libobjc.A.dylib`. C++ links use
`-nostdlib++` and import no `libc++`. Measured minos is iOS 10.0 for device and iOS 14.0 for
Simulator; the API floor is iOS 10.0

No tests, linked consumers, or probes were executed. Xcode 26.6 build 17F113 and iOS SDK 26.5
were used; this is below the repository's Xcode 27.x baseline. No passing CI workflow run had been
recorded at this validation point. Mainline run
[38066495166](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38066495166)
later passed the F28 C ABI gates as part of steps 235–275 on macOS 15 and Xcode 27. Linked probes
were not executed; this is not runtime or device evidence.
