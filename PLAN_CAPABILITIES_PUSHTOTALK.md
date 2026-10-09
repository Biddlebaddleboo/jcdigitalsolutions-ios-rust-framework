# PLAN_CAPABILITIES_PUSHTOTALK.md — D83: PushToTalk audit for row 084

## Scope

Audit row `084-cloud-accounts-communication-pushtotalk` for a public Rust-callable API, a non-prompting support or authorization query, API floor, host setup, and channel/push lifecycle constraints

This is an evidence and recommendation report only. It does not change the canonical matrix, global docs, CI, Cargo manifests, or source code

## Status and recommendation

Keep row 084 `X` in the canonical matrix until a useful Rust-owned PushToTalk feature is implemented and validated. The installed SDK exposes a public Objective-C class and protocols, and the generated binding project publishes `objc2-push-to-talk` 0.3.2. However, PushToTalk has no standalone `isSupported` or PushToTalk authorization-status query. The APIs found are for creating and operating the channel manager, handling its delegates and pushes, and reporting app-owned service status to the system UI

`PTChannelManager.activeChannelUUID` is a real read-only value, but only after the app creates and retains a channel manager with both delegates and its launch/restoration lifecycle. It reports the calling app's channel that is active in the system UI; it is not device support, entitlement, user authorization, microphone permission, APNs readiness, or a global PTT status query. Creating the manager is not a side-effect-free capability probe: initialization establishes the PTT lifecycle, can produce push-token callbacks, and is required to maintain restored channels and incoming pushes

`PTServiceStatus` is also not a query surface. It is a value the app sets for a channel to update the system UI about the app's own service state. The error values `PTInstantiationErrorMissingBackgroundMode`, `PTInstantiationErrorMissingPushServerEnvironment`, `PTInstantiationErrorMissingEntitlement`, and `PTInstantiationErrorInvalidPlatform` describe failed manager instantiation; they do not provide a separate non-prompting authorization or support API

No smaller honest status-only slice was found. A future implementation would need an explicitly scoped PTT channel lifecycle contract, callback ownership model, host configuration, microphone/audio handling, and push/server boundary; it must not be counted as generic PTT support merely because Rust can call the generated Objective-C declarations

## SDK and binding evidence

Inspection used Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5

- Framework path: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/PushToTalk.framework`
- Public module map: `PushToTalk.framework/Modules/module.modulemap`; public umbrella header: `PushToTalk.framework/Headers/PushToTalk.h`
- `PTChannelManager`, `PTChannelManagerDelegate`, `PTChannelRestorationDelegate`, the channel descriptor and push result classes, and error/status enums are public Objective-C declarations in the framework headers. The route is Objective-C, not a private Swift ABI or a public C function API
- The declarations start at iOS 16.0 and are unavailable on macOS, Mac Catalyst, tvOS, and watchOS. The SDK explicitly defines `PTInstantiationErrorInvalidPlatform` as unavailable on the Simulator or macOS devices. Apple also says PushToTalk services are unavailable to compatible iPhone and iPad apps running in visionOS
- `setAccessoryButtonEventsEnabled` starts at iOS 17.0; the optional incoming service-update push delegate method starts at iOS 17.2. These later APIs do not lower the framework's iOS 16.0 floor
- `PTChannelManager` exposes `activeChannelUUID` and operations such as `requestJoinChannelWithUUID:descriptor:`, `requestBeginTransmittingWithChannelUUID:`, `stopTransmittingWithChannelUUID:`, and `leaveChannelWithUUID:`. It exposes no read-only device support, PTT authorization, or system service status property
- `PTChannelManager` exposes `setServiceStatus:forChannelUUID:completionHandler:` only as a setter. The SDK says its default is `PTServiceStatusReady` and that the app should set a different value when its own underlying network is impaired
- The installed headers do not annotate the manager or delegate protocol as main-thread-only. The callback queue/thread contract must not be invented; a future Rust callback facade must audit object retention and callback context and marshal into a documented executor where required

The cached generated-framework catalog in `objc2` 0.6.5 lists `objc2-push-to-talk`; public upstream 0.3.2 docs expose `PTChannelManager`, its delegate and restoration traits, `PTServiceStatus`, `PTInstantiationError`, `PTPushResult`, and related public types. The current repository `Cargo.lock` and package manifests do not include this binding. No dependency was added or compiled for this audit

## Host configuration and user authorization

Apple's PushToTalk app setup lists these distinct host requirements:

- Enable the Push to Talk background mode, represented by `UIBackgroundModes` containing `push-to-talk`
- Add the Push to Talk capability and the Boolean entitlement `com.apple.developer.push-to-talk`
- Add the Push Notifications capability, which supplies the APNs environment entitlement required by manager instantiation
- Include `NSMicrophoneUsageDescription` with a purpose string for any app path that accesses the microphone

The SDK's `PTErrors.h` names missing background mode, APNs environment entitlement, and PushToTalk entitlement as separate instantiation failures. These failures show that linking the framework is not proof that the signed host can use it

There is no PTT authorization-status query in the inspected public headers or generated binding overview. Microphone recording permission is a separate AVFAudio permission: `AVAudioApplication.recordPermission` reports permission to record, and first audio input access prompts if the person has not decided. That value cannot establish that PushToTalk is supported, entitled, initialized, joined, or able to send a push

The SDK requires a foreground app to join a channel and says the join must follow explicit user interaction. Only one PushToTalk channel can be active on the system at a time. These are interaction and lifecycle requirements, not a PTT permission status API

## Channel, audio, and push lifecycle limits

- Apple requires the app to create the channel manager on launch; otherwise the system tears down channels and their ability to receive pushes. The host must retain the manager and implement both channel-manager and restoration delegates
- The restoration delegate may be called after app termination or device restart. The host must be able to provide the saved channel descriptor for a restored channel; callbacks and associated object lifetimes need a deliberate Rust/Objective-C ownership design
- After manager creation, PushToTalk supplies an ephemeral APNs token. The token is not active until a person joins a channel, is invalidated on leaving, and must not be cached in local storage. The host sends it to its service and must refresh its server state as the token/channel lifecycle changes
- PTT pushes use APNs push type `pushtotalk` and an `apns-topic` equal to the app bundle ID plus `.voip-ptt`. APNs delivery and the app's server are external dependencies; this framework alone does not supply a communications backend or audio transport
- For every incoming PTT push, the required delegate returns a nonnull `PTPushResult`; Apple says to return it promptly so the system can perform the requested action. This callback is not equivalent to parsing or delivering the audio payload in Rust
- The app supplies its own audio encoding and streaming. Before recording, it must wait for PushToTalk's audio-session activation delegate callback, and it must let the system manage session activation/deactivation priorities
- PushToTalk can wake the app for incoming audio while a channel is active, but it does not promise arbitrary background runtime, APNs delivery, network availability, or completion of app work
- A restricted-network fallback through Local Push Connectivity requires a separate Network Extension App Push Provider extension and host configuration. It is outside this row's bounded recommendation

## Why a status-only facade is not selected

There are three tempting but misleading mappings:

1. Treat `PTServiceStatus` as framework availability. It is an app-written channel UI status, not a system-read capability value
2. Treat `activeChannelUUID != nil` as general authorization or support. It means only that this app has a channel active in the system UI, after a channel-manager lifecycle has already been established
3. Call manager creation to infer permission from an error. That is lifecycle initialization with push and restoration callbacks and returns setup failures, not a documented authorization query. Its failure can reflect host signing or background-mode configuration and it is unsupported on the Simulator

The Objective-C API is callable from Rust through generated bindings, but this does not make a truthful portable or status-only contract. A later full slice could be viable only with an explicit Rust-owned delegate/callback contract, channel-restoration ownership, APNs server/token handling, and audio boundaries. That would be a separate implementation decision, not an inferred status feature

## Acceptance boundary

This audit establishes only:

- row 084 remains `X` pending a real bounded implementation
- PushToTalk has a public Objective-C API and an upstream generated Rust binding, but no discovered non-prompting PTT support or authorization query
- `PTServiceStatus` is set by the app; `activeChannelUUID` reflects only the app's active channel after manager initialization
- iOS 16.0 is the framework floor, while the API is explicitly unavailable on the Simulator and several other Apple platforms
- a useful implementation depends on a signed iOS host with PTT, background-mode, APNs, and microphone setup, plus a server and explicit channel lifecycle

This workstream does not claim channel creation, channel join, microphone permission, audio-session success, remote audio, push delivery, APNs authorization, PushToTalk entitlement approval, or native PTT parity

## Apple and binding references

- [PushToTalk framework](https://developer.apple.com/documentation/pushtotalk)
- [Creating a Push to Talk app](https://developer.apple.com/documentation/pushtotalk/creating-a-push-to-talk-app)
- [PTChannelManager](https://developer.apple.com/documentation/pushtotalk/ptchannelmanager?language=objc)
- [PTChannelManager activeChannelUUID](https://developer.apple.com/documentation/pushtotalk/ptchannelmanager/activechanneluuid?language=objc)
- [PTServiceStatus](https://developer.apple.com/documentation/pushtotalk/ptservicestatus)
- [PTInstantiationError](https://developer.apple.com/documentation/pushtotalk/ptinstantiationerror-swift.struct/code)
- [Push to Talk Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.push-to-talk)
- [UIBackgroundModes](https://developer.apple.com/documentation/bundleresources/information-property-list/uibackgroundmodes)
- [AVAudioApplication record permission](https://developer.apple.com/documentation/avfaudio/avaudioapplication/recordpermission-swift.property)
- [Request microphone recording permission](https://developer.apple.com/documentation/avfaudio/avaudioapplication/requestrecordpermission%28completionhandler%3A%29)
- [NSMicrophoneUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription)
- [APNs push type `pushtotalk`](https://developer.apple.com/documentation/usernotifications/sending-notification-requests-to-apns)
- [`objc2-push-to-talk` 0.3.2 generated binding](https://docs.rs/objc2-push-to-talk/0.3.2/objc2_push_to_talk/)
- Local SDK headers: `PushToTalk.framework/Headers/PushToTalk.h`, `PTChannelManager.h`, `PTChannelManagerDelegate.h`, `PTChannelRestorationDelegate.h`, and `PTErrors.h` under the framework path above
- Local binding catalog: `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_data.md`
- Current row record: `docs/capabilities/capability-status.json`, id `084-cloud-accounts-communication-pushtotalk`

## Audit record

Inspected the installed PushToTalk module map and public headers, Xcode version and iPhoneOS SDK version, cached generated-binding catalog, current Cargo manifests and lock, row 084's status record, upstream `objc2-push-to-talk` 0.3.2 docs, and Apple primary API, entitlement, background-mode, microphone, and APNs docs

No source, build, link, test, device, Simulator, entitlement, microphone prompt, push, or runtime probe was run for this audit. The row remains `X` until a separate workstream implements and validates a scoped Rust-owned PTT feature

## B179 follow-up: no standalone read-only PushToTalk query

Revalidated row 084 against the installed iOS 26.5 SDK declarations and Apple's current Objective-C
API documentation. The candidate `PTChannelManager.activeChannelUUID` has a truthful, narrow
meaning, but it is an instance property reachable only after the app creates and retains a manager
with channel-manager and restoration delegates. Apple requires manager creation at launch to
restore channels and receive pushes. This is operational lifecycle setup, not an isolated
capability query, so it is not suitable as a support or authorization snapshot for the Rust-only
facade.

`PTServiceStatus` also has meaningful named values, but `setServiceStatus:forChannelUUID:` is a
setter: Apple directs the app to report its own backend/network state to the system UI. The SDK
does not expose a getter. The instantiation failures for `PTInstantiationErrorMissingBackgroundMode`,
`PTInstantiationErrorMissingPushServerEnvironment`, `PTInstantiationErrorMissingEntitlement`, and
`PTInstantiationErrorInvalidPlatform` are reported only as part of channel-manager creation; they
do not form an independent support query and the invalid-platform error includes Simulator.
Calling manager creation solely to inspect an error would trigger the delegate and restoration
lifecycle the status-only contract excludes.

The installed public declarations are in
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/PushToTalk.framework/Headers/PTChannelManager.h`
and `PTErrors.h`. They confirm `channelManagerWithDelegate:restorationDelegate:completionHandler:`
as the manager factory, `activeChannelUUID` as the sole read-only manager status property, and
`setServiceStatus:forChannelUUID:completionHandler:` as a setter. The generated upstream
`objc2-push-to-talk` binding does not remove those instance/lifecycle requirements; the repository
does not depend on it. Therefore B179 adds no Rust binding or facade and row 084 remains `X`.

Primary API evidence: [PTChannelManager (Objective-C)](https://developer.apple.com/documentation/pushtotalk/ptchannelmanager?language=objc),
[activeChannelUUID](https://developer.apple.com/documentation/pushtotalk/ptchannelmanager/activechanneluuid?language=objc),
[Creating a Push to Talk app](https://developer.apple.com/documentation/pushtotalk/creating-a-push-to-talk-app),
[PTServiceStatus](https://developer.apple.com/documentation/pushtotalk/ptservicestatus),
and [PushToTalk](https://developer.apple.com/documentation/pushtotalk)

No code, dependency, build, link probe, test, app launch, Simulator run, device call, permission
prompt, channel operation, push operation, or live manager call was performed for B179
