# Swift Residual Alternatives — Pass 1

Research date: 2026-10-07

This pass examines several higher-value Swift-first frameworks at the **capability level** and asks whether older Objective-C/C APIs, public web services, or adjacent frameworks can eliminate some or all Swift ABI work.

The strongest conclusion is that several frameworks are **hybrid residuals**: only the newer account/configuration or system-integration layer is Swift-only, while substantial functional subsets remain available through Objective-C, C, or REST.

---

# MusicKit versus Apple Music API / MediaPlayer

## Executive finding

MusicKit is **not an all-or-nothing Swift requirement**.

A significant subset of Apple Music functionality can already be implemented without Swift ABI:

1. Apple Music catalog and user-library data are exposed through the public Apple Music API web service.
2. MediaPlayer exposes Objective-C playback controllers.
3. `MPMusicPlayerStoreQueueDescriptor` accepts Apple Music/store identifiers through an Objective-C API.
4. Legacy StoreKit exposes Objective-C Apple Music authorization, capability, and Music User Token APIs — but Apple now deprecates them and directs developers to MusicKit.

Therefore:

- catalog metadata/search: **R0/R3 over REST**;
- user-library REST requests: possible with a Music User Token, but obtaining the token through the current supported Apple-platform path is a **MusicKit residual**;
- Apple Music playback using known store IDs: substantial **R2** path remains through MediaPlayer;
- future-proof authorization/subscription/account integration: **S2 residual**.

## Apple Music API is a public web service

Apple documents the Apple Music API as a web service for:

- catalog albums, songs, artists, playlists, videos, stations;
- search;
- charts and recommendations;
- user library content;
- user ratings;
- recently played content;
- playlist creation/modification.

Reference:
https://developer.apple.com/documentation/applemusicapi

A Rust application can call the catalog endpoints directly using its normal HTTP stack.

Example catalog endpoint:
https://developer.apple.com/documentation/applemusicapi/get-a-catalog-playlist

### Catalog access

Catalog requests use a developer token. They do not intrinsically require Swift ABI.

For framework design, catalog access belongs in a normal Rust service/client layer rather than MusicKit ABI wrappers unless the in-process MusicKit model layer provides a compelling unique capability.

## Personalized Apple Music API access

Apple documents that personalized requests require a Music User Token.

Reference:
https://developer.apple.com/documentation/applemusicapi/user-authentication-for-musickit

Historically, Objective-C StoreKit exposed:

```objc
- requestUserTokenForDeveloperToken:completionHandler:
```

and authorization/capability methods on `SKCloudServiceController`.

Apple now marks these deprecated and says to use MusicKit.

References:
https://developer.apple.com/documentation/storekit/skcloudservicecontroller/requestusertoken(forDeveloperToken:completionHandler:)?language=objc
https://developer.apple.com/documentation/storekit/skcloudservicecontroller/requestauthorization(_:)?language=objc
https://developer.apple.com/documentation/storekit/skcloudservicecontroller/authorizationstatus()

### Consequence

The old Objective-C path may remain useful for compatibility research, but the framework should **not design a new long-term Apple Music integration around deprecated authorization/token APIs**.

A supported future-proof Rust implementation should eventually reach:

- `MusicAuthorization`;
- subscription/account capability state;
- MusicKit-managed Music User Token behavior;

through Swift ABI if no new public Objective-C/C replacement appears.

Classification:
- **REST catalog: R0/R3**
- **legacy account auth/token: R2 but deprecated**
- **modern account/auth integration: S2**

## Playback can remain Objective-C for many use cases

MediaPlayer is an Objective-C-capable framework for controlling music playback.

Apple exposes:

```objc
MPMusicPlayerController
MPMusicPlayerStoreQueueDescriptor
```

A store queue descriptor can be initialized from Apple Music/store IDs.

References:
https://developer.apple.com/documentation/mediaplayer/
https://developer.apple.com/documentation/mediaplayer/mpmusicplayerstorequeuedescriptor?language=objc
https://developer.apple.com/documentation/mediaplayer/mpmusicplayerstorequeuedescriptor/storeids?language=objc

This is important because MusicKit's Swift `SystemMusicPlayer` is not automatically required merely to play known Apple Music items.

### Framework recommendation

Support native MediaPlayer playback first.

Only implement MusicKit playback ABI if it provides a required behavior that MediaPlayer cannot provide or if Apple deprecates the relevant MediaPlayer path.

## User library playback versus user-library REST

MediaPlayer can search/play content from the user's media library after authorization.

Apple Music API can retrieve the user's iCloud Music Library with a Music User Token.

These are overlapping but not identical surfaces.

The framework should expose them as separate capability layers rather than forcing everything through MusicKit's Swift model types.

## MusicKit residual ABI shape

Where MusicKit is actually necessary, relevant types include generic Swift value types such as:

```swift
MusicCatalogResourceRequest<MusicItemType>
```

and Swift classes such as `SystemMusicPlayer`.

References:
https://developer.apple.com/documentation/musickit

Likely ABI needs:
- Swift generics;
- Decodable/model values;
- async;
- arrays/pages;
- authorization status enums;
- possibly Swift actor/main-actor restrictions.

### Priority

**P1/P2 hybrid**, not ahead of StoreKit 2 or Translation.

---

# AdAttributionKit versus AdServices

## Executive finding

AdAttributionKit provides a distinct privacy-preserving install/reengagement attribution mechanism. AdServices does **not** replace it.

AdServices remains native Objective-C for Apple's attribution token, while AdAttributionKit postback management is Swift-value/async based.

## AdServices native path

Apple exposes Objective-C attribution-token APIs through AdServices.

This remains an R2 capability for use cases that specifically need Apple's attribution token.

## AdAttributionKit capability

For the advertised app, Apple requires conversion-value updates to begin and maintain the postback conversion window.

Apple exposes:

```swift
struct Postback
```

with async throwing static methods such as:

```swift
static func updateConversionValue(...) async throws
```

Reference:
https://developer.apple.com/documentation/adattributionkit/postback

The framework supports:
- install attribution;
- reengagement attribution;
- privacy-preserving winning/nonwinning postbacks;
- StoreKit-rendered/custom-rendered ads.

Reference:
https://developer.apple.com/documentation/adattributionkit/receiving-ad-attributions-and-postbacks

## Recommendation

Do not try to substitute AdServices for AdAttributionKit.

Instead:

- expose AdServices separately through native ObjC;
- classify AdAttributionKit's app-side conversion update API as **S2**;
- keep it optional because many applications do not participate in ad-network attribution flows.

Likely ABI needs:
- static Swift methods;
- Swift structs/enums;
- async/throws;
- strings/integers/bools.

This is a relatively contained category-A Swift-call target, but not a general framework priority.

### Priority

**P2**.

---

# WorkoutKit versus HealthKit

## Executive finding

WorkoutKit exposes a capability that HealthKit does not replace: creating workout compositions and scheduling/syncing them into Apple's Workout app on Apple Watch.

HealthKit remains the native route for health/workout data, but **WorkoutKit is a real Swift residual for Workout-app integration**.

Apple describes WorkoutKit as enabling apps to:

- create structured workouts;
- preview workout plans;
- open plans in the Workout app;
- schedule workouts;
- sync scheduled workouts to Apple Watch.

Reference:
https://developer.apple.com/documentation/workoutkit

## Core Swift types

Apple exposes:

```swift
struct WorkoutPlan
final class WorkoutScheduler
```

`WorkoutScheduler` uses async methods for authorization and schedule management.

References:
https://developer.apple.com/documentation/workoutkit/workoutplan
https://developer.apple.com/documentation/workoutkit/workoutscheduler

## Useful serialization surface

`WorkoutPlan` exposes:

```swift
init(from: Data) throws
var dataRepresentation: Data
```

This is potentially important.

It creates a natural binary/data boundary for storing or transferring already-created workout plans.

However:

> Do not assume the `dataRepresentation` format is a public independently constructible wire format.

Research must determine whether Apple documents the representation sufficiently for Rust to construct it directly. If not, it is useful only after a WorkoutKit plan already exists.

## HealthKit overlap

HealthKit should remain the native Objective-C path for:

- reading/writing health samples;
- workout records;
- workout sessions and health data.

WorkoutKit should be implemented only for the unique composition/scheduling/Workout-app capability.

## Likely ABI needs

- Swift structs/enums describing workouts/goals/alerts;
- associated nested value types;
- arrays;
- DateComponents;
- async;
- throws;
- scheduler class.

No app-defined Swift protocol type appears necessary for the ordinary plan/scheduling path, making WorkoutKit primarily a **category-A S2** consumer of Apple-defined Swift values.

### Priority

**P2**, after generic Layer-1 Swift value/async support.

---

# ProximityReader versus PassKit/CoreNFC

## Executive finding

ProximityReader is a genuine unique capability for Tap to Pay on iPhone and merchant-side customer engagement. PassKit/CoreNFC do not provide an equivalent unrestricted lower-level path.

Apple exposes:

```swift
class PaymentCardReader
```

with Swift async methods and Swift request/result value types.

Reference:
https://developer.apple.com/documentation/proximityreader/paymentcardreader

## Capability

ProximityReader allows supported merchant apps to:

- configure Tap to Pay on iPhone;
- accept contactless payment cards;
- read loyalty cards;
- support Store and Forward modes;
- support Tap to Share/customer engagement.

Reference:
https://developer.apple.com/documentation/proximityreader

## Access restrictions

This is not a general-purpose NFC API.

Apple requires:

- an organization-level developer account;
- a Tap to Pay on iPhone managed entitlement;
- Apple approval;
- coordination with a participating Level 3-certified payment service provider;
- a separate distribution entitlement for TestFlight/App Store distribution.

Reference:
https://developer.apple.com/documentation/proximityreader/setting-up-the-entitlement-for-tap-to-pay-on-iphone

## ABI shape

Core operations include:

```swift
func prepare(using: PaymentCardReader.Token) async throws
    -> PaymentCardReaderSession
```

and Swift value types such as:

- `PaymentCardTransactionRequest`;
- `PaymentCardVerificationRequest`;
- `PaymentCardReadResult`;
- options/tokens;
- Store-and-Forward values.

### Classification

**S2, specialized P2**.

The framework can support it once generic Swift class/value/async primitives exist, but entitlement/commercial restrictions mean it should not drive the foundational ABI design.

---

# VisionKit: framework-level "Swift-only" classification is misleading

## Executive finding

VisionKit is strongly **hybrid**.

The current objc2 framework catalog classifies VisionKit as unsupported/Swift-only at the framework level, but Apple's public documentation shows important VisionKit capabilities are explicitly Objective-C compatible.

This proves why symbol-level research is required.

## DataScannerViewController

Apple declares:

```swift
@MainActor @objc
class DataScannerViewController
```

Reference:
https://developer.apple.com/documentation/visionkit/datascannerviewcontroller

It can be configured, presented and controlled as a UIKit view controller.

The delegate path is the best Rust-first route.

Apple also exposes:

```swift
var recognizedItems: AsyncStream<[RecognizedItem]>
```

Reference:
https://developer.apple.com/documentation/visionkit/datascannerviewcontroller/recognizeditems

That AsyncStream is a Swift convenience/observation path. The existence of the delegate means the framework does **not** need to implement AsyncStream merely for data scanning.

## Document camera

Apple exposes:

```objc
@protocol VNDocumentCameraViewControllerDelegate <NSObject>
```

with standard Objective-C callbacks.

Reference:
https://developer.apple.com/documentation/visionkit/vndocumentcameraviewcontrollerdelegate?language=objc

Therefore document scanning is plainly R2.

## Recommendation

Treat VisionKit capability-by-capability:

- document camera: **R2**
- DataScanner controller/delegate: **R2**
- AsyncStream recognized items: **D** initially; use delegate
- any newer Swift-only recognition/value surfaces: research only if they provide unique functionality

### Result

VisionKit should be removed from the broad "needs Swift ABI" mental model for common scanning use cases.

---

# LiveCommunicationKit versus CallKit

## Executive finding

LiveCommunicationKit adds real modern capabilities beyond CallKit, but ordinary VoIP integration does not require it.

Apple defines:

```swift
final class ConversationManager
```

with Swift value/class types and async operations.

Reference:
https://developer.apple.com/documentation/livecommunicationkit/conversationmanager

## Ordinary VoIP

CallKit already provides Objective-C/native integration for incoming/outgoing VoIP call UI and system calling behavior.

For a Rust-first app that only needs ordinary VoIP, CallKit remains the lower-complexity R2 path.

## LiveCommunicationKit-specific value

Apple's current framework includes richer conversation semantics and, importantly, supports apps acting as the default calling app/default dialer.

Apple documents:

- a default calling app may use LiveCommunicationKit or CallKit for VoIP;
- a **default dialer app** uses LiveCommunicationKit for cellular-network conversations;
- `StartCellularConversationAction` / `TelephonyConversationManager` require the Default Dialer App entitlement;
- testing default dialer behavior currently has EU account/device-location requirements.

Reference:
https://developer.apple.com/documentation/livecommunicationkit/preparing-your-app-to-be-the-default-dialer-app

## Runtime API

`ConversationManager` exposes:
- Swift `Configuration`;
- arrays of `Conversation`;
- arrays of `ConversationAction`;
- Swift protocol delegate;
- `perform([ConversationAction]) async throws`;
- async incoming-conversation reporting.

Reference:
https://developer.apple.com/documentation/livecommunicationkit/conversationmanager

## Delegate requirement

`ConversationManagerDelegate` is a Swift protocol.

A pure-Rust implementation may therefore require:
- Layer-1 calling support for manager/actions;
- Layer-2 Swift protocol-conformance support for a Rust delegate.

This makes it more complex than a simple async class wrapper.

## Recommendation

Split capability:

- ordinary VoIP/call integration: **CallKit R2**
- default dialer/cellular integration and LCK-only features: **S2 / Layer 1+2**

### Priority

**P2 specialized**, especially because default dialer functionality is entitlement/region constrained.

---

# LockedCameraCapture versus ordinary camera access

## Executive finding

LockedCameraCapture provides a unique lock-screen/Control Center/Action-button capture-extension integration. It is **not needed for ordinary camera access**, which remains AVFoundation/UIKit native.

The unique extension is tightly coupled to modern Swift extension-scene and App Intents infrastructure.

## Core types

Apple exposes:

```swift
final class LockedCameraCaptureSession
```

with async methods such as:
- invalidating temporary session content;
- opening the containing app.

Reference:
https://developer.apple.com/documentation/lockedcameracapture/lockedcameracapturesession

But the extension itself is declared through:

```swift
LockedCameraCaptureExtension
LockedCameraCaptureUIScene
```

and a SwiftUI-style scene body/content closure.

Reference:
https://developer.apple.com/documentation/lockedcameracapture/lockedcameracaptureextension/body-swift.property

## App Intents dependency

Apple's integration uses:

```swift
protocol CameraCaptureIntent : SystemIntent
```

which inherits from AppIntent and has a Codable/Sendable associated `AppContext`.

Reference:
https://developer.apple.com/documentation/appintents/cameracaptureintent

This means a fully pure-Rust locked-camera extension likely depends on the App Intents Layer-3 work already identified.

## Security constraints

Apple documents major sandbox restrictions for the locked capture extension:

- no network access;
- no App Group shared-container access;
- extension data container is ephemeral and erased after suspension/transfer;
- an active camera view/hardware capture interaction is required;
- captured session content moves through dedicated session directories.

Reference:
https://developer.apple.com/documentation/lockedcameracapture/creating-a-camera-experience-for-the-lock-screen

These are domain constraints, not language-runtime problems.

## Recommendation

- ordinary camera: **AVFoundation R2**
- lock-screen capture session methods: potentially **S2**
- defining a complete capture extension/CameraCaptureIntent: **S3-ish**, coupled to App Intents and Swift extension-scene metadata
- do not make this a core framework milestone

### Priority

**P3 specialized**.

---

# Cross-framework conclusions

## 1. Prefer partial native coverage over framework-wide Swift classification

The following all turned out to have meaningful native subsets:

- MusicKit ecosystem: REST + MediaPlayer
- VisionKit: @objc scanner/document camera controllers/delegates
- LiveCommunicationKit use case: CallKit substitutes for ordinary VoIP
- LockedCameraCapture use case: AVFoundation substitutes for ordinary camera

The framework should expose the native subset first and add Swift ABI only for the unique remaining capability.

## 2. Deprecation status matters more than technical availability

A deprecated Objective-C path is useful evidence and may remain operational, but it is not a suitable long-term foundation for a new framework.

Examples:
- SKCloudServiceController Apple Music auth/token APIs
- StoreKit 1 purchase APIs

The research matrix should distinguish:
- native and supported;
- native but deprecated;
- Swift-only current replacement.

## 3. Entitlement-gated Swift APIs should not shape Layer 1

ProximityReader and default-dialer LiveCommunicationKit are technically valuable, but access restrictions make them poor choices for the first ABI proof.

StoreKit 2 and Translation remain better Layer-1 validation targets.

## 4. Delegate/callback alternatives can eliminate Swift async streams

VisionKit demonstrates a recurring pattern:

```text
Swift convenience:
AsyncStream<[RecognizedItem]>

Native alternative:
@objc controller + delegate callbacks
```

When both exist, the Rust framework should use the native delegate path unless the stream semantics provide unique functionality worth the ABI complexity.

## Revised priority after this pass

1. StoreKit 2
2. Translation
3. Foundation Models basic text session
4. MusicKit modern authorization/account pieces if needed
5. WorkoutKit
6. AdAttributionKit
7. ProximityReader when entitlement-bearing test environment exists
8. LiveCommunicationKit unique/default-dialer pieces
9. LockedCameraCapture only after App Intents/tooling support
10. VisionKit common scanning remains native and drops out of Swift priority
