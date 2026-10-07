# Master Apple Framework Census

Research date: 2026-10-07

## Purpose

This file is the durable top-level census for the framework research.

It answers three questions:

1. Which Apple framework families already have a public Objective-C/C path and therefore should not drive Swift ABI work?
2. Which framework families are currently Swift-only or substantially Swift-first?
3. In what order should the remaining Swift-first families be researched?

This is a **framework-family census**, not proof that every symbol in a native framework is Objective-C/C compatible. High-value hybrid frameworks still require symbol-level research.

## External coverage baseline

The current objc2 0.6.4 generated-framework catalog provides a useful independent coverage map. It lists a large set of Apple frameworks with generated Objective-C/C Rust bindings and separately lists frameworks it considers unsupported, often explicitly because they are Swift-only.

Reference:
https://docs.rs/objc2/latest/objc2/topics/about_generated/list/index.html

Important interpretation:

- "objc2 supported" is strong evidence that a substantial public Objective-C/C header surface exists.
- It is not proof that every newer API in that framework is Objective-C-compatible.
- "objc2 unsupported: Swift-only" is a strong signal for residual research, but Apple documentation remains authoritative for current SDK behavior.
- C-heavy frameworks can be excellent fits for this project even when objc2 intentionally does not wrap them.

---

# Native-first framework families already covered by objc2

The current objc2 catalog includes generated bindings for the following Apple framework families.

For this project, these default to **native-first research status**: use objc2/public C before considering Swift ABI.

## Very common / core app capabilities

- Foundation
- UIKit
- CoreFoundation
- CoreGraphics
- QuartzCore
- CoreText
- CoreImage
- CoreData
- CoreLocation
- CoreBluetooth
- CoreMotion
- CoreNFC
- AVFoundation
- AVFAudio
- AVKit
- AudioToolbox
- UserNotifications
- AuthenticationServices
- LocalAuthentication
- Security
- WebKit
- MapKit
- Contacts
- ContactsUI
- Photos
- PhotosUI
- EventKit
- EventKitUI
- CloudKit
- StoreKit (legacy/Objective-C surface only; StoreKit 2 remains residual)
- NetworkExtension
- BackgroundTasks
- PassKit
- GameKit
- GameController
- CoreML
- Vision
- Metal
- MetalKit
- MetalPerformanceShaders
- MetalPerformanceShadersGraph
- PDFKit
- QuickLook
- QuickLookThumbnailing
- ReplayKit
- SafariServices
- Speech
- NaturalLanguage
- LinkPresentation
- UniformTypeIdentifiers

## Communication / calling / sharing

- CallKit
- PushKit
- PushToTalk
- Messages
- MessageUI
- SharedWithYou
- SharedWithYouCore
- IdentityLookup
- IdentityLookupUI
- WatchConnectivity
- ExternalAccessory
- NearbyInteraction
- MultipeerConnectivity (deprecated for new work; use Network)
- CarPlay

## Device/system/privacy

- DeviceCheck
- AccessorySetupKit
- AppTrackingTransparency
- AdServices
- AdSupport
- SensitiveContentAnalysis
- SensorKit
- CoreTelephony
- ThreadNetwork
- HomeKit
- HealthKit
- HealthKitUI
- SafetyKit
- SystemConfiguration
- MetricKit
- OSLog
- ScreenTime

## Media/graphics/creative

- CoreAudio
- CoreAudioKit
- CoreAudioTypes
- CoreMedia
- CoreVideo
- VideoToolbox
- MediaPlayer
- MediaToolbox
- MediaAccessibility
- Cinematic
- PHASE
- SoundAnalysis
- ShazamKit
- SpriteKit
- SceneKit
- PencilKit
- ModelIO
- ImageIO
- Symbols
- Accessibility

## Extensions/storage/document infrastructure

- FileProvider
- FileProviderUI
- ExtensionKit
- AppClip
- BackgroundAssets
- BrowserEngineCore
- BrowserEngineKit
- DeviceDiscoveryExtension
- FSKit

## Other generated/native framework families

- ARKit
- AVRouting
- AutomaticAssessmentConfiguration
- CFNetwork
- ClassKit
- ClockKit
- CompositorServices
- CryptoTokenKit
- DataDetection
- GameplayKit
- JavaScriptCore
- MediaExtension
- MediaSetup
- MLCompute
- NotificationCenter
- ScreenCaptureKit
- ServiceManagement
- Social
- TVServices
- TVUIKit
- UserNotificationsUI
- VideoSubscriberAccount

Some entries are platform-specific or less relevant to iOS, but their presence reinforces the broad native ABI surface available to Rust.

---

# C-centric families that may be even better Rust targets

objc2 intentionally leaves some C-heavy frameworks unsupported because its translator focuses on Objective-C-style frameworks.

That does **not** mean they are poor fits for this project.

Examples:

- Accelerate
- Network
- low-level Security/CoreFoundation surfaces
- libSystem/Darwin APIs
- portions of AudioToolbox/CoreAudio
- system compression/math/simd C APIs

For this project:

> A public C API is often preferable to a Swift wrapper because Rust can call it directly with ordinary native FFI.

Therefore "not supported by objc2" must never be equated with "requires Swift."

---

# Confirmed or strongly indicated Swift-first residual families

The current objc2 catalog explicitly classifies many framework families as Swift-only. Current Apple docs add newer frameworks not yet reflected in that catalog.

The list below is filtered toward iOS-relevant capabilities.

## Tier A — research first: broad capability value

### StoreKit 2

Status: **S2 / P0**

Why:
- modern Product/Transaction API;
- old Objective-C purchase API deprecated;
- generic values;
- async/throws;
- AsyncSequence.

Dedicated research:
- `STOREKIT2_SWIFT_ABI_FEASIBILITY.md`

### AppIntents

Status: **S3 / P0-P1**

Why:
- system discovery;
- Swift protocols/associated types;
- compiler-generated static metadata;
- async perform.

Dedicated research:
- `APP_INTENTS_METADATA_FEASIBILITY.md`

### Translation

Status: **S2 / P1**

Why:
- direct non-UI `TranslationSession`;
- async/throws;
- Swift values;
- useful contained runtime-ABI proof.

Dedicated research:
- `TRANSLATION_SWIFT_ABI_FEASIBILITY.md`

### FoundationModels

Status: **S2 initially / P1**

Why:
- on-device generative model capability;
- plain session/text use appears possible without custom Generable type;
- structured generation later adds macros/protocol metadata.

Dedicated classification:
- `SWIFT_FIRST_RESIDUALS_PASS2.md`

### AlarmKit

Status: **S2 / P1-P2**

Why:
- async manager API;
- generic AlarmConfiguration;
- AlarmMetadata protocol;
- AsyncSequence updates.

Dedicated classification:
- `NEWER_FRAMEWORK_SWEEP_PASS1.md`

### MusicKit

Status: **unresearched Swift residual / likely S2 / P1-P2**

Reason for priority:
- common consumer-media capability;
- objc2 labels framework Swift-only.

Research questions:
- which Apple Music operations still have MusicKit/MediaPlayer Objective-C alternatives;
- token/subscription/catalog/playback split;
- whether MediaPlayer covers enough playback/control to avoid Swift.

### AdAttributionKit

Status: **unresearched Swift residual / P1-P2**

Research questions:
- attribution registration/postback capability;
- whether AdServices covers common non-marketplace use;
- value/async ABI surface.

---

# Tier B — important but specialized or requires Rust-defined Swift types

### ActivityKit

Status: **S2 / P2**

Why:
- `Activity<Attributes>` generic;
- app-defined `ActivityAttributes`;
- Codable/Hashable associated ContentState;
- async update/streams.

Dedicated classification:
- `SWIFT_FIRST_RESIDUALS_PASS2.md`

### GroupActivities

Status: **S2 / P2**

Why:
- app-defined `GroupActivity`;
- Codable conformance;
- generic GroupSession;
- AsyncSequence sessions.

Dedicated classification:
- `SWIFT_FIRST_RESIDUALS_PASS2.md`

### FamilyControls

Status: **hybrid S2 / P2**

Why:
- authorization has a simpler callback path;
- application/category/domain selection tokens are Swift types;
- SwiftUI picker often used;
- entitlement restricted.

### DeviceActivity

Status: **S2 / P2**

Why:
- Swift structs for center/names/schedules/events.

### ManagedSettings / ManagedSettingsUI

Status: **S2 / P2**

Why:
- Swift settings/token ecosystem;
- specialized parental-control entitlement use.

### FinanceKit / FinanceKitUI

Status: **S2 / P2**

Why:
- Swift async/value API;
- managed entitlement limits deployment.

### WorkoutKit

Status: **unresearched Swift residual / P2**

Research questions:
- workout schedule/model construction;
- HealthKit fallback/overlap;
- app-defined values/protocols;
- sync with Apple Watch.

### CarKey

Status: **unresearched Swift residual / P2**

Research questions:
- entitlement/access restrictions;
- Secure Element/UWB/NFC dependencies;
- whether common consumer apps can use it at all.

### SecureElementCredential

Status: **unresearched Swift residual / P2**

Research questions:
- entitlement/issuer restrictions;
- hardware/secure-element lifecycle;
- overlap with PassKit/CoreNFC/Security.

### ProximityReader

Status: **unresearched Swift residual / P2**

Research questions:
- Tap to Pay/contactless merchant capability;
- entitlement/commercial restrictions;
- whether PassKit provides lower-level Objective-C alternatives.

### LockedCameraCapture

Status: **unresearched Swift residual / P2-P3**

Research questions:
- capture extension lifecycle;
- app/extension registration;
- AVFoundation fallback;
- whether framework offers unique lock-screen integration only.

### LiveCommunicationKit

Status: **unresearched Swift residual / P2**

Research questions:
- relationship to CallKit/PushToTalk;
- whether it adds required modern call state beyond Objective-C frameworks.

---

# Tier C — likely useful but lower priority

### CoreTransferable

Status: **D/P3**

Most transfer capabilities already have:
- pasteboard;
- drag/drop;
- share UI;
- document/file APIs.

Implement only for interoperability requirements.

### TipKit

Status: **P3**

Rust can implement its own eligibility/state rules and UIKit hints. Only pursue if system TipKit integration itself matters.

### JournalingSuggestions

Status: **P3**

UI-heavy Swift API and entitlement/specialized use.

### WeatherKit

Status: **D for Swift ABI**

Apple exposes REST API, so the Swift framework is not required for capability access.

### SwiftData

Status: **D/P3**

Persistence is already available through Rust/SQLite/Core Data. SwiftData macro/model interoperability is optional.

### Combine

Status: **D**

Reactive abstraction, not a unique system capability.

### CryptoKit

Status: **D by default**

Prefer:
- Rust crypto;
- Security/SecKey/Keychain for system/hardware key capabilities.

Research only genuine Secure Enclave gaps.

### Charts

Status: **D/P3**

UI-heavy and replaceable through UIKit/custom/native rendering if ever needed.

### SwiftUI / SwiftUICore

Status: **D/P3**

Not a core target. UIKit is the framework's primary native UI path.

### WidgetKit

Status: **P2-P3**

System widget capability is valuable, but rendering is heavily SwiftUI-oriented. Research non-UI provider/discovery pieces separately.

### ImagePlayground

Status: **hybrid**

Current system view controller/delegate is explicitly `@objc`; deprecated programmatic ImageCreator is Swift-only.

Prefer native UIKit presentation.

### VisionKit

Status: **unresearched hybrid candidate**

objc2 labels framework Swift-only, but many scanner/document UI capabilities historically expose UIKit controllers/delegates.

Needs symbol-level verification before assigning Swift work.

### RealityKit / RealityFoundation

Status: **P3 unless spatial/AR app**

ARKit/Metal/SceneKit provide native alternatives for many non-RealityKit requirements.

### RoomPlan

Status: **P3 specialized**

Research only if room-scanning apps need it.

### TabletopKit

Status: **P3 specialized**

Spatial/visionOS-oriented.

### DockKit

Status: **P3 specialized**

Accessory/camera tracking use.

---

# Lower-frequency / enterprise / platform-specific Swift residuals

These should not drive the general iOS framework core, but remain in the census so they are not forgotten.

- Assignables
- AutomatedDeviceEnrollment
- ContactProvider
- CoreHID
- CreateML
- CreateMLComponents
- DeveloperToolsSupport
- ExtensionFoundation
- LightweightCodeRequirements
- ManagedApp
- ManagedAppDistribution
- MarketplaceKit
- MatterSupport
- RealityFoundation
- SecureElementCredential
- TabularData
- TranslationUIProvider

Some are macOS/enterprise/developer-tool oriented or entitlement-limited.

---

# Deprecated/obsolete paths to avoid

The objc2 catalog also flags numerous old frameworks that should not receive new wrapper investment.

Examples:

- AddressBook / AddressBookUI -> Contacts
- AssetsLibrary -> PhotoKit
- AudioUnit legacy surface -> newer AudioToolbox/AVAudio paths where appropriate
- CalendarStore -> EventKit
- GLKit/OpenGL/OpenGLES/AGL -> Metal
- MediaLibrary -> PhotoKit
- MobileCoreServices -> CoreServices/UniformTypeIdentifiers
- MultipeerConnectivity -> Apple currently recommends Network for new peer networking
- Twitter -> Social
- iAd -> AdServices

The framework should prefer current supported replacements.

---

# Research progress by capability importance

## Completed native elimination or initial classification

- UIKit/Foundation
- HTTP networking
- preferences
- keychain/security
- filesystem
- notifications
- location
- Bluetooth
- motion
- background tasks
- camera
- audio/video
- Photos
- Contacts
- authentication/passkeys
- WebKit
- CloudKit
- Core ML
- Vision
- Metal
- maps
- Core Data
- NFC
- HealthKit core
- CallKit
- PushKit
- App Attest / DeviceCheck
- EventKit
- GameKit
- NetworkExtension/VPN
- NearbyInteraction
- HomeKit
- ExternalAccessory
- PDF/QuickLook/MessageUI
- AccessorySetupKit
- SensitiveContentAnalysis
- ImagePlayground hybrid surface
- StoreKit 2
- Translation
- App Intents
- Foundation Models initial classification
- ActivityKit initial classification
- GroupActivities initial classification
- FinanceKit initial classification
- AlarmKit initial classification
- FamilyControls/DeviceActivity/ManagedSettings initial classification
- SwiftData/Observation/Combine/CoreTransferable/WeatherKit/CryptoKit high-level disposition

## Next high-value research queue

1. MusicKit versus MediaPlayer/native alternatives.
2. AdAttributionKit versus AdServices.
3. WorkoutKit versus HealthKit.
4. ProximityReader versus PassKit/payment-related native surfaces.
5. VisionKit symbol-level audit.
6. LiveCommunicationKit versus CallKit/PushToTalk.
7. LockedCameraCapture versus AVFoundation.
8. CarKey/SecureElementCredential access restrictions and native alternatives.
9. WidgetKit non-rendering/system-facing pieces.
10. MatterSupport versus Matter/HomeKit native paths.
11. lower-frequency Swift-only frameworks.

---

# Architectural conclusion from the master census

The total Apple SDK surface is **far more native-accessible than Swift-only**.

The expected architecture should therefore remain:

```text
ordinary app capability
  |
  +--> pure Rust                  R0
  |
  +--> public C ABI              R1
  |
  +--> Objective-C / objc2       R2
  |
  +--> thin Rust ergonomics      R3
  |
  '--> only if none above works:
       Swift ABI/tooling residual
```

Swift interoperability should be a **small optional subsystem**, not the foundation of the iOS framework.

The master census also shows why framework-level labels are insufficient: newer Apple frameworks increasingly contain mixtures of @objc-compatible controllers/delegates and Swift-only value types. Research must continue at the capability/symbol level for high-priority hybrid frameworks.
