# High-Value Residual Audit — Pass 2

Research date: 2026-10-07

This pass continues the elimination-first audit for several high-value frameworks that remained unresolved in the master census.

## Summary

| Framework | Result | Class | Priority |
|---|---|---|---|
| MusicKit | Split capability: catalog/library data largely avoidable via Apple Music API REST; legacy/native playback exists via MediaPlayer; modern MusicKit playback/model remains Swift-only | Hybrid / S2 only where needed | P1-P2 |
| AdAttributionKit | Genuine Swift value/async API | S2 | P1-P2 |
| WorkoutKit | Genuine Swift workout-model/scheduling API, but HealthKit covers workout tracking itself | S2 | P2 |
| ProximityReader | Genuine Swift async payment-reader API; heavily entitlement/provider gated | S2 | P2 |
| VisionKit | Hybrid; DataScannerViewController is explicitly @objc | R2 + residual Swift config values | Native-first |
| LiveCommunicationKit | Genuine Swift class/value/async API; CallKit remains Objective-C alternative for ordinary VoIP | S2 | P2 |
| LockedCameraCapture | System integration itself is Swift-heavy and extension/scene based; actual camera capture remains AVFoundation/UIKit native | S2/S3-ish | P2-P3 |
| CarKey | Swift-first and entitlement/MFi-automaker restricted | S2 | P3/general framework |
| SecureElementCredential | Swift actor/async API, but UIKit integration exists; highly entitlement/ABR restricted | S2 | P3/general framework |
| WidgetKit non-rendering | WidgetCenter is still Swift-only; provider/rendering remains Swift protocol/generic heavy | S2/S3 | P2-P3 |
| MatterSupport | Hybrid: extension request handler is @objc, while request values/perform path are Swift value/async | R2 + S2 | P2 |

# MusicKit

## Finding: do not treat all Apple Music capability as a Swift requirement

MusicKit itself is strongly Swift-oriented:

- `MusicCatalogSearchRequest` is a Swift struct.
- Search response is async/throws.
- catalog item types conform to Swift protocols such as `MusicCatalogSearchable`.
- `ApplicationMusicPlayer` and `SystemMusicPlayer` are Swift classes.

References:
https://developer.apple.com/documentation/musickit/musiccatalogsearchrequest
https://developer.apple.com/documentation/musickit/applicationmusicplayer
https://developer.apple.com/documentation/musickit/systemmusicplayer

However, Apple separately exposes the **Apple Music API** as a web service for catalog and user-library data.

It supports:
- catalog search;
- albums/artists/songs/playlists/stations;
- user library access;
- ratings;
- recommendations;
- history;
- playlist modification;
- favorites and replay data.

Reference:
https://developer.apple.com/documentation/applemusicapi

### Architectural split

#### Catalog/library data

Prefer the Apple Music API through Rust networking where practical.

This moves a large portion of MusicKit's model/request surface from:

```text
Rust -> Swift ABI -> MusicKit model layer
```

to:

```text
Rust -> HTTPS -> Apple Music API
```

This is not a latency optimization—the network dominates either way—but it avoids Swift ABI/model complexity and is portable.

Do not use REST merely to avoid a tiny native call if the local framework has uniquely better semantics; use it because Apple explicitly offers it as a first-class public service.

#### Playback

MediaPlayer still exposes `MPMusicPlayerController`, including queueing by Apple Music store identifiers.

Apple documents `setQueue(with storeIDs: [String])`.

References:
https://developer.apple.com/documentation/mediaplayer/mpmusicplayercontroller
https://developer.apple.com/documentation/mediaplayer/mpmusicplayercontroller/setqueue(with:)-8x6xb

Therefore basic playback control may not require MusicKit ABI.

Modern MusicKit playback types may still provide capabilities or queue/model ergonomics unavailable through MediaPlayer; research those only when a concrete application needs them.

### Classification

- catalog/search/library metadata: **D for Swift ABI** where Apple Music API is sufficient;
- ordinary music-library/system playback: investigate **R2 MediaPlayer** first;
- modern MusicKit-only playback/model capability: **S2**.

This is another major reduction in early Swift ABI scope.

# AdAttributionKit

## Finding

AdAttributionKit is genuinely Swift-first.

Apple exposes:

```swift
struct AppImpression
```

with:

```swift
init(compactJWS: String) async throws
func beginView() async throws
func endView() async throws
func handleView() async throws
func handleTap() async throws
```

and:

```swift
struct Postback
static func updateConversionValue(...) async throws
```

References:
https://developer.apple.com/documentation/adattributionkit/appimpression
https://developer.apple.com/documentation/adattributionkit/postback

## ABI surface

This is a relatively clean category-A Swift ABI problem:

- Swift struct construction;
- String;
- URL;
- UInt64/Int/Bool;
- async/throws;
- enums/struct configuration.

No app-defined Swift protocol type appears necessary for basic advertiser/publisher flows.

### Native adjunct

Click-through validation uses `UIEventAttributionView`, a UIKit-level component.

The eventual framework can keep ad UI native and use Swift ABI only for the attribution value operations.

## Priority

**P1-P2**.

It is a useful post-Translation ABI target because it exercises Swift structs and async static/instance methods without requiring custom protocol conformances.

# WorkoutKit

## Finding

WorkoutKit is not required to **track** workouts.

Apple's HealthKit retains Objective-C APIs such as:
- `HKWorkoutConfiguration`;
- `HKWorkoutSession`.

References:
https://developer.apple.com/documentation/healthkit/hkworkoutconfiguration
https://developer.apple.com/documentation/healthkit/hkworkoutsession?language=objc

WorkoutKit adds a different capability:
- create structured workout compositions;
- preview them;
- schedule them;
- sync them to the Workout app on Apple Watch;
- export/open plans.

Apple exposes Swift value models including:
- `SingleGoalWorkout`;
- `PacerWorkout`;
- `CustomWorkout`;
- `WorkoutPlan`;
- `WorkoutScheduler`.

Reference:
https://developer.apple.com/documentation/workoutkit

## Framework strategy

Split:
- live workout tracking and health samples -> **HealthKit/objc2 R2**;
- structured Apple Watch Workout-app plans/scheduling -> **WorkoutKit S2**.

Do not make WorkoutKit a prerequisite for generic fitness support.

## Priority

**P2**.

# ProximityReader / Tap to Pay

## Finding

ProximityReader is a genuine Swift async API.

`PaymentCardReader.prepare(using:)` is:

```swift
func prepare(using token: PaymentCardReader.Token)
    async throws -> PaymentCardReaderSession
```

A session exposes async methods such as:

```swift
func readPaymentCard(...) async throws -> PaymentCardReadResult
func capturePIN(...) async throws -> PaymentCardReadResult
func cancelRead() async throws -> Bool
```

References:
https://developer.apple.com/documentation/proximityreader/paymentcardreader/prepare(using:)
https://developer.apple.com/documentation/proximityreader/paymentcardreadersession

## Deployment restrictions

Apple requires:
- coordination with a participating Level 3-certified payment service provider;
- Tap to Pay entitlement approval.

Therefore this API should not drive general ABI implementation despite being a clean category-A Swift target.

## Important distinction from PassKit

PassKit's `PKPaymentAuthorizationController` is Objective-C and handles **Apple Pay payment authorization for a customer using the device**.

ProximityReader makes the iPhone act as a **merchant contactless reader**.

They are not interchangeable capabilities.

Reference:
https://developer.apple.com/documentation/passkit/pkpaymentauthorizationcontroller?language=objc

## Priority

**P2**, after StoreKit/Translation/general Swift async/value support.

# VisionKit

## Finding

VisionKit is **hybrid**, and an important previously unresolved API can be removed from the Swift-only list.

Apple declares:

```swift
@MainActor @objc
class DataScannerViewController
```

It uses a delegate and native view-controller presentation.

Reference:
https://developer.apple.com/documentation/visionkit/datascannerviewcontroller

This provides:
- live text recognition;
- data-in-text scanning;
- machine-readable code scanning.

## Residual issue

Some configuration/value types accepted by the initializer are Swift-native nested/value types.

Therefore:
- presentation/lifecycle/delegate is R2;
- constructing all modern configuration values may need limited Swift ABI or manual binding support.

## Framework strategy

Do not label VisionKit as globally Swift-only.

For scanning apps:
1. investigate the @objc initializer/selectors exported by the SDK;
2. use direct UIKit controller/delegate path where available;
3. fall back to AVFoundation + Vision for custom scanners before implementing heavy Swift ABI solely for VisionKit convenience.

Classification: **native-first hybrid**.

# LiveCommunicationKit

## Finding

LiveCommunicationKit is genuinely Swift-first.

Apple exposes:

```swift
final class ConversationManager
```

with Swift value configuration and async methods including:

```swift
func perform([ConversationAction]) async throws
func reportNewIncomingConversation(...) async throws
static func reportNewIncomingVoIPPushPayload(...) async throws
```

The delegate is a Swift class protocol.

References:
https://developer.apple.com/documentation/livecommunicationkit/conversationmanager
https://developer.apple.com/documentation/livecommunicationkit/conversationmanagerdelegate

## Existing native alternative

CallKit remains Objective-C and supports ordinary VoIP call integration through `CXProvider` and `CXProviderDelegate`.

Apple explicitly documents that default calling apps may use **CallKit or LiveCommunicationKit**.

References:
https://developer.apple.com/documentation/callkit
https://developer.apple.com/documentation/callkit/cxprovider?language=objc

## Strategy

For ordinary VoIP:
- prefer CallKit R2 first.

Research LiveCommunicationKit only for capabilities uniquely required for:
- newer conversation model;
- default dialer/calling-app integration;
- cellular forwarding/system integration beyond CallKit.

## Priority

**P2**.

# LockedCameraCapture

## Finding

The **camera work** itself does not require Swift.

Apple's LockedCameraCapture guide explicitly tells developers to use AVFoundation capture APIs and even demonstrates UIKit `UIImagePickerController` within the capture experience.

Reference:
https://developer.apple.com/documentation/LockedCameraCapture/Creating-a-camera-experience-for-the-Lock-Screen

However, the **system extension integration** is Swift-heavy:

- `LockedCameraCaptureExtension` protocol;
- `LockedCameraCaptureExtensionScene` protocol;
- generic `LockedCameraCaptureUIScene<Content: View>`;
- `LockedCameraCaptureSession`;
- `LockedCameraCaptureManager.sessionContentUpdates` AsyncSequence.

References:
https://developer.apple.com/documentation/LockedCameraCapture
https://developer.apple.com/documentation/lockedcameracapture/lockedcameracaptureuiscene
https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturesession

## Architectural split

- camera pipeline: **AVFoundation/UIKit R2**;
- special Lock Screen/Control Center capture extension entry/scene: **S2/S3-ish**.

The scene body is tied to SwiftUI/AppExtension scene machinery, making this a poor early target.

## Priority

**P2-P3** unless lock-screen camera integration becomes a specific product requirement.

# CarKey

## Finding

CarKey exposes Swift classes/structs for remote keyless vehicle control.

More importantly, Apple restricts the entitlement:

`com.apple.developer.carkey.session`

to automakers enrolled in the MFi Program.

Reference:
https://developer.apple.com/documentation/carkey

## Strategy

CarKey is a real Swift ABI target but a poor general-framework priority because ordinary third-party apps cannot simply use it.

Do not spend early ABI time on CarKey.

## Priority

**P3 for general framework**.

# SecureElementCredential

## Finding

The core entrypoint is:

```swift
actor CredentialSession
```

It exposes state and an `AsyncStream` of events.

Apple provides UIKit and SwiftUI transaction UI extensions.

Use requires:
- Secure Element Credential entitlement;
- Apple Business Register applet registration;
- platform eligibility.

References:
https://developer.apple.com/documentation/secureelementcredential
https://developer.apple.com/documentation/secureelementcredential/credentialsession

## Strategy

Core management is S2+ because actor/async-stream semantics are part of the API.

UIKit transaction UI may reduce SwiftUI dependence, but it does not eliminate the Swift actor/session ABI.

Because deployment is restricted, do not let this shape the core ABI layer.

## Priority

**P3 general / P1 only for an eligible contactless-credential product**.

# WidgetKit non-rendering pieces

## Finding

Even the non-rendering management API is Swift-first.

`WidgetCenter` is a Swift class that:
- reloads timelines;
- queries current configurations.

`WidgetInfo` is a Swift struct.

Reference:
https://developer.apple.com/documentation/widgetkit/widgetcenter

Widget creation itself adds:
- `TimelineEntry` protocol;
- generic `Timeline`;
- provider protocols;
- Widget/SwiftUI rendering.

Reference:
https://developer.apple.com/documentation/widgetkit/widgets-and-complications-collection

## Strategy

There may be value in supporting only:
- `WidgetCenter.reloadAllTimelines`;
- reload by kind;
- current-configuration queries;

as a small Layer-1 Swift ABI package before attempting Rust-authored widget extensions.

Actual widget definitions remain Layer 2/3/4 work.

## Priority

- WidgetCenter management: **P2**;
- Rust-authored widget extension/rendering: **P3**.

# MatterSupport

## Finding

MatterSupport is hybrid.

Apple exposes:

```swift
@objc
class MatterAddDeviceExtensionRequestHandler
```

which third-party extensions subclass.

Reference:
https://developer.apple.com/documentation/mattersupport/matteradddeviceextensionrequesthandler

But the initiating request is a Swift value with async execution:

```swift
MatterAddDeviceRequest
func perform() async throws
```

Reference:
https://developer.apple.com/documentation/mattersupport/matteradddevicerequest

## Strategy

Split:
- extension callback handler: likely R2 via Rust-defined Objective-C subclass;
- request/topology/value construction and async `perform`: S2.

Do not force the @objc extension path through Swift.

## Priority

**P2**, specialized smart-home ecosystem support.

# Revised queue after this pass

## Promote for early Layer-1 ABI testing

After StoreKit and Translation:

1. AdAttributionKit — clean Swift structs + async.
2. Foundation Models plain text.
3. WidgetCenter management — small system-facing Swift class API.
4. MusicKit only for capability gaps not covered by Apple Music API/MediaPlayer.

## Keep specialized

5. WorkoutKit.
6. ProximityReader.
7. MatterSupport request path.
8. LiveCommunicationKit.
9. AlarmKit.
10. DeviceActivity/ManagedSettings.

## Defer due heavy type/build/UI requirements

11. ActivityKit custom attributes.
12. GroupActivities custom activity types.
13. App Intents metadata/type generation.
14. LockedCameraCapture extension scene.
15. full WidgetKit extension/rendering.

## Defer due entitlement/ecosystem restrictions

16. CarKey.
17. SecureElementCredential.
18. FinanceKit where managed entitlement unavailable.

# Cross-cutting conclusion

This pass again demonstrates that the optimal architecture is **capability-level**, not framework-level.

Examples:

- MusicKit catalog data can often disappear into Apple Music REST.
- Music playback can sometimes stay in Objective-C MediaPlayer.
- VisionKit's headline scanner view controller is explicitly @objc.
- MatterSupport's extension handler is @objc even though request execution is Swift.
- LockedCameraCapture requires Swift for the special extension shell, not for the camera pipeline.
- LiveCommunicationKit is Swift, but ordinary VoIP still has CallKit.

The Swift ABI subsystem should therefore remain modular and opt-in. A normal Rust iOS application should pay none of its complexity or runtime cost unless it actually uses one of the residual capabilities.
