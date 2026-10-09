# Documentation Policy

Shared validation commands, report scope, CI coverage, and environment limits live in [Validation and Tooling](VALIDATION.md).

The capability support matrix lives in [`capabilities/README.md`](capabilities/README.md); the
local-authentication contract, portable byte/UTF-8 contract, and portable connectivity snapshot,
opt-in Keychain C ABI, notification-response C ABI, iOS transfer C ABI, iOS clipboard C ABI,
iOS share C ABI, iOS preferences C ABI, iOS MediaPlayer status C ABI, and C++17 layer have separate [authentication guide](capabilities/authentication.md),
[portable data guide](capabilities/data.md),
[portable connectivity guide](capabilities/connectivity.md),
[Keychain guide](bindings/secure-storage.md), [response ABI guide](bindings/notification-responses.md),
[iOS transfer guide](bindings/ios-transfer.md), [iOS clipboard guide](bindings/ios-clipboard.md),
[iOS share guide](bindings/ios-share.md), [iOS preferences guide](bindings/ios-preferences.md),
[iOS MediaPlayer status ABI guide](bindings/ios-media-library-status.md),
[iOS motion guide](ios/motion.md), and [C++ layer guide](bindings/cpp.md)

The iOS LocalAuthentication backend has a separate [iOS authentication guide](ios/authentication.md).

The App Tracking Transparency status-only contract and iOS backend have separate
[privacy authorization](capabilities/privacy-authorization.md) and
[iOS tracking authorization](ios/tracking-authorization.md) guides.

The DeviceCheck and App Attest support snapshot has separate
[portable](capabilities/device-integrity.md) and [iOS](ios/device-integrity.md) guides.

The Watch Connectivity support snapshot has separate
[portable](capabilities/watch-connectivity.md) and [iOS](ios/watch-connectivity.md) guides.

The ExternalAccessory presence snapshot has separate
[portable](capabilities/accessory.md) and [iOS](ios/external-accessory.md) guides.

ReplayKit availability and the built-in SoundAnalysis classifier-recognition snapshot have separate
[portable ReplayKit](capabilities/replaykit.md), [iOS ReplayKit](ios/replaykit.md),
[portable SoundAnalysis](capabilities/sound-analysis.md), and [iOS SoundAnalysis](ios/sound-analysis.md)
guides.

The point-in-time other-audio playback snapshot has separate [portable](capabilities/other-audio.md)
and [iOS](ios/other-audio.md) guides; it does not implement Now Playing metadata or media control.

The bounded HDR playback-eligibility snapshot has separate
[portable playback](capabilities/playback.md) and [iOS playback](ios/playback.md) guides; it does
not create a player, inspect an asset, or start playback.

The MessageUI mail/text availability and SharedWithYou software-support bits have separate
[MessageUI](ios/message-ui-support.md) and [SharedWithYou](ios/shared-with-you-support.md) guides.
They expose no message composition, account, highlight, or collaboration operation.

The platform-exclusive Apple Pay capability status has separate [portable-scope](capabilities/apple-pay.md)
and [iOS availability](ios/apple-pay-availability.md) guides; it does not inspect cards or process payments.

The portable FourCC and hardware-decode result have a separate [video codec guide](capabilities/video-codec.md).
The VideoToolbox hardware-decode query has a separate [iOS guide](ios/videotoolbox.md); it does not
encode, process media frames, or reserve hardware resources.

The ReplayKit availability snapshot has separate
[portable](capabilities/replaykit.md) and [iOS](ios/replaykit.md) guides.

The reusable portable native-control slice and UIKit backend have separate [portable UI](capabilities/ui.md)
and [iOS UI](ios/ui.md) guides.

The `ios-files` crate's synchronous Foundation coordination extension has a separate [file-coordination guide](ios/file-coordination.md). The [security-scoped URL access guide](ios/security-scoped-access.md) covers balancing an already-issued Foundation scope; it does not select URLs or add file I/O.

The `ios-connectivity` crate's one-shot public Network.framework path snapshot has a separate
[iOS connectivity guide](ios/connectivity.md).

The `ios-data` crate's explicit raw-byte copies to and from immutable Core Foundation data have a
separate [iOS data guide](ios/data.md).

The `ios-url` crate's strict Foundation URL value adapter has a separate [iOS URL guide](ios/url.md).

The portable finite rational media-time value and its CoreMedia adapter have separate [portable media](capabilities/media.md)
and [iOS media](ios/media.md) guides.

The portable system-font metric contract and its CoreText adapter have separate
[font metrics](capabilities/text-metrics.md) and [iOS CoreText metrics](ios/text-metrics.md) guides.

The portable ImageIO metadata contract and metadata-only iOS adapter have separate
[image metadata](capabilities/image-metadata.md) and [iOS ImageIO](ios/image-metadata.md) guides.

The portable Photos authorization contract and PhotoKit backend have separate
[Photos](capabilities/photos.md) and [iOS Photos](ios/photos.md) guides.

The portable UIKit background-execution lease and iOS adapter have separate
[background-execution](capabilities/background-execution.md) and
[iOS background-execution](ios/background-execution.md) guides.

The portable Contacts authorization contract and iOS Contacts backend have separate
[Contacts](capabilities/contacts.md) and [iOS Contacts](ios/contacts.md) guides.

The portable Calendar authorization contract and EventKit backend have separate
[Calendar](capabilities/calendar.md) and [iOS Calendar](ios/calendar.md) guides.

The portable HealthKit authorization-request contract and public iOS backend have separate
[HealthKit](capabilities/health-authorization.md) and [iOS HealthKit](ios/health-authorization.md) guides.

The portable Bluetooth authorization and central-discovery contracts have separate
[Bluetooth](capabilities/bluetooth.md), [iOS Bluetooth](ios/bluetooth.md), and
[iOS Bluetooth discovery](ios/bluetooth-discovery.md) guides.

The platform-exclusive HTTPS web-view contract and bounded `WKWebView` adapter have separate
[portable web](capabilities/web.md) and [iOS web](ios/web.md) guides.

The iCloud Drive identity-presence contract and Foundation adapter have separate
[portable iCloud identity](capabilities/icloud-drive-identity.md) and
[iOS iCloud identity](ios/icloud-drive-identity.md) guides.

The CloudKit account-status snapshot contract and public CloudKit adapter have separate
[portable account-status](capabilities/cloudkit-account-status.md) and
[iOS account-status](ios/cloudkit-account-status.md) guides.

The SafetyKit Crash Detection device-support bit has a separate [iOS guide](ios/safetykit.md); it
does not claim event entitlement, authorization, delivery, or emergency response.

The ARKit world-tracking support value and iOS configuration query have separate
[portable](capabilities/arkit.md) and [iOS](ios/arkit.md) guides.

The Game Center local-player authentication-status contract and iOS backend have separate
[portable](capabilities/gamekit-status.md) and [iOS](ios/game-center-status.md) guides.

The Core ML compute-device availability snapshot has a separate [iOS guide](ios/core-ml-device-status.md);
it does not load a model, run inference, or guarantee a particular model can run.

The portable Vision text-recognition revision-support value and iOS query have separate
[portable](capabilities/vision.md) and [iOS](ios/vision.md) guides; they do not create a request
or process an image.

The iOS Speech authorization-status snapshot has a separate [iOS guide](ios/speech-status.md); it
does not request permission or process audio.

The English Natural Language contextual-model asset snapshot has a separate
[iOS guide](ios/natural-language-status.md); it does not load a model or accept text.

The deprecated legacy StoreKit purchase-ability snapshot has a separate
[iOS guide](ios/storekit-status.md); it does not add a purchase or transaction API.

The StoreKit 2 `AppStore.canMakePayments` status query has a separate
[iOS guide](ios/storekit2-status.md); it does not retrieve products or process purchases.

The host-presented SafariServices HTTPS controller has a separate
[iOS guide](ios/safari.md); SafariServices owns browser content and navigation.

The iOS-only Accelerate vDSP vector-add operation has a separate
[iOS guide](ios/accelerate.md); it adds no portable math contract or performance claim.

The iOS-only CommonCrypto SHA-256 wrapper has a separate [iOS guide](ios/crypto.md); it adds no
portable crypto facade or Rust replacement claim.

The iOS-only ModelIO extension-status query has a separate [iOS guide](ios/modelio-status.md); it
does not load or parse a file or claim rendering support.

The iOS-only MPS preferred-device presence query has a separate [iOS guide](ios/mps-status.md); it
does not submit GPU work or establish MPS operation support.

The portable P-256 key-suitability contract and its iOS Security backend have separate
[capability](capabilities/key-support.md) and [iOS](ios/key-support.md) guides; the query does not
verify signatures or persist keys.

The iOS-only MediaPlayer library authorization snapshot has a separate
[guide](ios/media-library-status.md); it does not request access or provide Apple Music service
support. The opt-in F11 C ABI has a separate [guide](bindings/ios-spritekit.md); its opaque handle
wraps only detached node position and keeps SpriteKit objects inside Rust.

The portable finite SpriteKit node-position contract and iOS detached `SKNode.position` adapter have
separate [capability](capabilities/spritekit-node-position.md) and
[iOS](ios/spritekit-node-position.md) guides; they do not create scenes, views, or rendered content.
The CallKit active-call count/state snapshot has an [iOS guide](ios/call-observer.md) and an opt-in
[C ABI guide](bindings/ios-call-observer.md); it exposes no identifiers or call-control surface.
The finite portable map geometry and MapKit backend have separate
[capability](capabilities/mapkit.md) and [iOS](ios/mapkit.md) guides. The opt-in F12–F14 C ABI
guides are [CallKit](bindings/ios-call-observer.md), [MapKit](bindings/ios-maps.md), and
[ClassKit](bindings/ios-classkit-deep-link.md); the ClassKit C API accepts a caller-borrowed
`NSUserActivity` pointer only for its marker query.
The ClassKit incoming-activity marker has a separate [iOS guide](ios/classkit-deep-link.md); it
does not access Schoolwork assignment data. The [WeatherKit audit](../PLAN_CAPABILITIES_WEATHERKIT.md)
and [RealityKit audit](../PLAN_CAPABILITIES_REALITYKIT.md) record why their current rows remain
unsupported under the scoped Rust/Objective-C surface.
The [NetworkExtension/VPN audit](../PLAN_CAPABILITIES_NETWORK_EXTENSION.md) records the remaining scope beyond B79's integrated Personal VPN profile-status slice; see the [B79 plan](../PLAN_IOS_VPN_STATUS.md), [capability guide](capabilities/vpn.md), and [iOS guide](ios/vpn-status.md).
The [PushToTalk audit](../PLAN_CAPABILITIES_PUSHTOTALK.md) and
[CarPlay audit](../PLAN_CAPABILITIES_CARPLAY.md) record the entitlement, push/audio, and host scene
lifecycle boundaries that keep those rows unsupported.
The [ExtensionKit/Foundation audit](../PLAN_CAPABILITIES_EXTENSIONKIT.md) and
[ContactProvider audit](../PLAN_CAPABILITIES_CONTACTPROVIDER.md) record the Swift-only and host
lifecycle boundaries for rows 100 and 102. The [FileProvider capability guide](capabilities/file-provider.md),
[B75 plan](../PLAN_IOS_FILEPROVIDER.md), and [G78 gate](../PLAN_VALIDATION_IOS_FILEPROVIDER.md)
cover a narrower caller-app registered-domain query; compile/Clippy/rustdoc and link/import gates passed,
while probes were inspected but not executed. The [BrowserEngineKit](../PLAN_CAPABILITIES_BROWSERENGINEKIT.md)
and [ManagedApp/Distribution](../PLAN_CAPABILITIES_MANAGEDAPP.md), [MarketplaceKit](../PLAN_CAPABILITIES_MARKETPLACEKIT.md), [MatterSupport](../PLAN_CAPABILITIES_MATTERSUPPORT.md), and [SecureElementCredential](../PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md), [CarKey](../PLAN_CAPABILITIES_CARKEY.md), and [ProximityReader](../PLAN_CAPABILITIES_PROXIMITYREADER.md), and [LockedCameraCapture](../PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md) audits cover Swift ABI, approval, setup, entitlement, and process/MDM boundaries for rows 101 and 103–109.
The [extension bundle metadata](../PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md),
[WidgetKit](../PLAN_CAPABILITIES_WIDGETKIT.md), [ActivityKit](../PLAN_CAPABILITIES_ACTIVITYKIT.md),
and [App Intents](../PLAN_CAPABILITIES_APP_INTENTS.md) audits cover rows 110–113. B77 now reads
one caller-selected extension point from a `.appex` bundle; it does not generate build metadata or
implement App Intents. ActivityKit compiler signatures match, but owned-value linking remains
unproven. See the [extension metadata guide](ios/extension-support.md).

The [CallKit D76 plan](../PLAN_CAPABILITIES_CALLKIT.md),
[MapKit D77 plan](../PLAN_CAPABILITIES_MAPKIT.md),
[ClassKit D81 plan](../PLAN_CAPABILITIES_CLASSKIT.md),
[CallKit validation](../PLAN_VALIDATION_IOS_CALL_OBSERVER.md),
[MapKit validation](../PLAN_VALIDATION_IOS_MAPKIT.md), and
[ClassKit validation](../PLAN_VALIDATION_IOS_CLASSKIT.md),
[CallKit C ABI validation](../PLAN_VALIDATION_C_ABI_CALL_OBSERVER.md), and
[MapKit C ABI validation](../PLAN_VALIDATION_C_ABI_MAPS.md), and
[ClassKit C ABI validation](../PLAN_VALIDATION_C_ABI_CLASSKIT.md) record precise scope and
evidence limits. The opt-in [F14 plan](../PLAN_BINDINGS_CLASSKIT.md) documents pointer and
availability semantics; [F15](../PLAN_BINDINGS_LOCATION.md) defines Location operation readiness,
poll, cancel, and destroy semantics, with its focused gate passed and probes unexecuted.
The [F16 FileProvider C ABI](../PLAN_BINDINGS_FILEPROVIDER.md) defines borrowed identifier and owned
result-buffer semantics; its host/device/Simulator gate passed, with link probes not executed.
The [F17 Vision C ABI](../PLAN_BINDINGS_F17.md) defines a synchronous revision-membership query; its
focused host/device/Simulator gate passed and does not execute consumers or probes.
The [F18 ProximityReader C ABI](../PLAN_BINDINGS_F18.md) defines only the Tap to Pay device-model
predicate; it does not establish payment readiness.
The [F19 CommonCrypto C ABI](../PLAN_BINDINGS_F19.md) exposes one iOS SHA-256 operation only,
the [F20 ModelIO C ABI](../PLAN_BINDINGS_F20.md) exposes only the extension-support Boolean,
the [F21 Sign in with Apple C ABI](../PLAN_BINDINGS_F21.md) exposes one prior-user query with a
single arbitrary-queue completion and no cancellation. The [F22 Accelerate C ABI](../PLAN_BINDINGS_F22.md)
exposes only equal-length f32 vector addition through vDSP, and the [F23 key-support C ABI](../PLAN_BINDINGS_F23.md)
exposes only P-256 public-key suitability for ECDSA/SHA-256 message verification. Their focused gates do not execute
consumers or probes. The [P-256 key-support guide](bindings/ios-key-support.md) and the [F24 MPS status C ABI](../PLAN_BINDINGS_F24.md)
define their narrow query bounds; the [MPS status guide](bindings/ios-mps-status.md) records preferred-device presence only. The [Sign in with Apple credential-state guide](ios/sign-in-with-apple-status.md)
documents B78's entitlement-scoped query.
The [F25 VideoToolbox C ABI plan](../PLAN_BINDINGS_F25.md) and [guide](bindings/ios-videotoolbox.md)
expose only the B50 hardware-decode predicate for one caller-supplied FourCC; they do not create a
decoder session, process media, or guarantee decoder resources. Its local link gates passed, but no
consumer/probe was executed and no passing CI workflow run is recorded.
The [F26 camera-device-status C ABI plan](../PLAN_BINDINGS_F26.md) and
[guide](bindings/ios-camera-device-status.md) expose only B62's current default video-device
presence Boolean; they do not query authorization, capture setup, or readiness. Device and
Simulator probe minima are 10.0 and 14.0; the runtime-guarded API floor is iOS 4.0. Its local gates
passed, but no consumer/probe was executed and no passing CI workflow run is recorded.
The [F27 Core ML status C ABI plan](../PLAN_BINDINGS_F27.md) and
[guide](bindings/ios-core-ml-status.md) expose only B56's nonempty compute-device-list Boolean.
The API floor is iOS 17.0; measured link minos are device 11.0 and Simulator 14.0. Its static/build
and native link/import gates passed; no tests, consumers, or probes were run.
The [F28 Speech status C ABI plan](../PLAN_BINDINGS_F28.md) and
[guide](bindings/ios-speech-status.md) expose only B58's signed authorization code, preserving
unknown raw values. The API floor and device link minos are iOS 10.0; the Simulator minos is 14.0.
Its C11/C++17 link/import gates passed; no consumer or probe was executed. No permission request,
prompt, recognizer, audio, or recognition flow is in scope.
The [F29 Natural Language status C ABI plan](../PLAN_BINDINGS_F29.md) and
[guide](bindings/ios-natural-language-status.md) expose only B59's English contextual-model asset
state as one of five fixed `uint32_t` codes. The API floor and device/Simulator link minos are iOS
17.0. Its C11/C++17 link/import gates passed; no consumer or probe was executed. No model load, text,
vector, or asset request is in scope.

The [F30 extension metadata C ABI plan](../PLAN_BINDINGS_F30.md) and
[guide](bindings/ios-extension-support.md) expose one caller-selected `.appex`
`NSExtension.NSExtensionPointIdentifier` read through B77. The backend API floor is iOS 4.0;
measured link minos are device 12.0 and Simulator 14.0. Host/device/Simulator C11/C++17 build and
link/import gates passed in the integrated checkout; consumers and probes were not executed. No
extension load or launch is in scope.

The [F31 RoomPlan support C ABI plan](../PLAN_BINDINGS_F31.md) and
[guide](bindings/ios-roomplan-status.md) expose only B61's `RoomCaptureSession.isSupported`
Boolean, with an iOS 16.0 deployment floor. No RoomPlan session, frame access, permission request,
UI, or scan is in scope. Host C/C++ imports only libSystem; device/Simulator imports RoomPlan and
libSystem at minos 16.0. Both static/build and link/import gates passed in the integrated checkout;
no passing CI workflow run is recorded.

The [F32 StoreKit 2 status C ABI plan](../PLAN_BINDINGS_F32.md) and
[guide](bindings/ios-storekit2-status.md) expose only B63's `AppStore.canMakePayments` Boolean.
The API/symbol floor is iOS 15.0; device/Simulator C/C++ link gates pass with weak StoreKit and
`libSystem.B.dylib` imports at minos 10.0/14.0. The absent-symbol fallback was not runtime-checked
below the API floor, and no consumer or probe was executed.
The [F33 Game Center status C ABI plan](../PLAN_BINDINGS_F33.md) and
[guide](bindings/ios-game-status.md) expose one momentary local-player authentication snapshot.
The SDK API floor is iOS 4.1; device/Simulator C/C++ imports are Foundation, GameKit,
`libSystem.B.dylib`, and `libobjc.A.dylib` at minos 10.0/14.0. The signed app needs the
`com.apple.developer.game-center` entitlement. No auth flow, live player query, consumer, or probe
was run; no passing CI workflow is recorded.
The [FamilyControls status feasibility audit](../PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md)
records why row 074 remains unsupported. The [Foundation Models](../PLAN_CAPABILITIES_FOUNDATION_MODELS.md),
[DeviceActivity](../PLAN_CAPABILITIES_DEVICE_ACTIVITY.md), [TipKit](../PLAN_CAPABILITIES_TIPKIT.md),
and [AdAttributionKit/AdServices](../PLAN_CAPABILITIES_AD_ATTRIBUTION.md) audits record separate
Swift ABI, authorization-semantics, presentation, and network-operation boundaries. The
[Thread audit](../PLAN_CAPABILITIES_THREAD.md) records the entitlement and preferred-network scope.
The [DockKit audit](../PLAN_CAPABILITIES_DOCKKIT.md) records its Swift ABI and async-sequence boundary.

The portable RoomPlan device-support value and iOS `RoomCaptureSession.isSupported` query have
separate [capability](capabilities/roomplan.md) and [iOS](ios/roomplan-status.md) guides; they do
not start a scan or access camera/LiDAR frames.

The iOS-only ProximityReader device-model predicate has a separate
[status guide](ios/proximity-reader-status.md) and [B76 plan](../PLAN_IOS_PROXIMITYREADER.md); it
does not establish Tap to Pay payment readiness.

The iOS-only default camera video-device presence query has a separate
[guide](ios/camera-device-status.md); it does not request permission, create a capture session, or
access media.

The portable camera/microphone authorization-status contract and AVFoundation adapter have separate
[media authorization](capabilities/media-authorization.md) and
[iOS media authorization](ios/media-authorization.md) guides.

The portable NFC reader-support snapshot and Core NFC adapter have separate
[NFC](capabilities/nfc.md) and [iOS NFC](ios/nfc.md) guides.

The portable Nearby Interaction capability-query contract and iOS adapter have separate
[Nearby Interaction](capabilities/nearby-interaction.md) and
[iOS Nearby Interaction](ios/nearby-interaction.md) guides.

The portable Metal default-device presence contract and iOS adapter have separate
[Metal](capabilities/metal.md) and [iOS Metal](ios/metal.md) guides.

The Watch Connectivity session-support query has separate
[portable](capabilities/watch-connectivity.md) and [iOS](ios/watch-connectivity.md) guides.

## Purpose

Documentation is part of the framework implementation.

Codex and other contributors must develop documentation continuously as the codebase grows. Do not defer documentation until after implementation.

## Documentation audiences

The project needs documentation for three different readers:

1. **Application developer**
   - how to use the API;
   - examples;
   - platform support;
   - errors;
   - async/cancellation;
   - performance-relevant behavior.

2. **Framework maintainer**
   - architecture;
   - internal invariants;
   - dependency seams;
   - packed-data layouts;
   - ownership/threading;
   - unsafe contracts;
   - backend design.

3. **Executor/reviewer**
   - why a design exists;
   - what was benchmarked;
   - Apple parity evidence;
   - known gaps;
   - validation commands.

Do not mix these audiences into one giant document when separate focused docs are clearer.

## Public API documentation

Every meaningful public Rust item should have useful rustdoc.

Document:
- purpose;
- semantics;
- errors;
- ownership/lifetime if non-obvious;
- thread affinity;
- cancellation;
- platform availability;
- complexity/cost when important;
- native escape behavior;
- examples.

Prefer examples that compile in CI where practical.

Do not expose internal packing details in ordinary public docs unless developers need them for ABI/serialization compatibility.

## Capability documentation

Each capability should eventually have a focused guide describing:

- what is portable;
- what is platform-specific;
- supported backends;
- availability/permissions/entitlements;
- high-level usage;
- lifecycle;
- errors;
- async/cancellation;
- native escape hatch;
- performance model;
- dependency/backend implementation choices;
- parity status if Rust replaces an Apple implementation.

## Architecture documentation

Update architecture docs in the same change when modifying:
- crate/module boundaries;
- portable contracts;
- backend selection;
- stable C ABI;
- ownership;
- async model;
- no_std guarantees;
- dependency substitution;
- Swift ABI approach;
- unsafe assumptions;
- public compatibility guarantees.

## Internal representation documentation

Compact internal representations should document enough for safe maintenance without leaking them into the public abstraction.

For packed words/tables include:
- field/bit allocation;
- range/capacity;
- sentinel/reserved values;
- overflow/wrap behavior;
- atomicity;
- alignment;
- endian/serialization assumptions if applicable;
- why the layout is performance-motivated.

## Dependency documentation

For each major third-party dependency record:
- why it is used;
- exact surface relied upon;
- why a small internal implementation is not preferable;
- direct and meaningful transitive costs;
- types that cross boundaries, ideally none;
- optional/default features enabled/disabled;
- no_std implications;
- replacement seam;
- runtime/binary/build cost where material;
- security/ABI/standards reasons for preferring the dependency when applicable.

The goal is that a future maintainer can both replace the dependency without rediscovering the architecture and understand why the dependency was worth adding in the first place.

## Apple parity documentation

When Rust replaces Apple library functionality, document:
- exact Apple API used as reference;
- supported semantic subset;
- intentional differences;
- OS-version differences;
- parity test location;
- benchmark location;
- why Rust is the default;
- conditions where Apple remains preferred.

## Performance documentation

Performance-sensitive changes should record:
- benchmark scenario;
- input sizes;
- hardware/OS/toolchain;
- build profile;
- comparison baseline;
- measured result;
- allocations/copies where relevant;
- limitations.

Do not preserve marketing-style claims without reproducible evidence.

## Examples

Examples should emphasize the ergonomic facade.

A developer should see:

```rust
let location = Location::current().await?;
```

not internal:
- bit masks;
- Objective-C selectors;
- JNI objects;
- Swift metadata;
- callback registries.

Advanced/native escape examples belong in separate sections.

## Documentation freshness

A change is incomplete when it changes behavior but leaves docs describing the old behavior.

Review documentation during final diff review.

At minimum check:
- rustdoc;
- README/guide;
- architecture docs;
- parity/testing docs;
- performance docs;
- platform availability.

## Research versus normative documentation

Keep exploratory findings in `docs/research/`.

Once a decision becomes an architectural rule, promote it into normative docs such as:
- `AGENTS.md`;
- `docs/ARCHITECTURE.md`;
- `docs/API_DESIGN.md`;
- `docs/PERFORMANCE.md`;
- `docs/TESTING_AND_PARITY.md`.

Research notes may contain uncertainty. Normative docs must state the adopted rule clearly.

## Documentation quality bar

Good documentation should let a smaller implementation model or a new maintainer:
- understand the design without broad repository exploration;
- know which files/modules own an invariant;
- know what must not change;
- reproduce validation;
- understand why a dependency or Apple backend remains;
- distinguish public semantics from internal representation.

Documentation is successful when it reduces rediscovery and makes architectural drift harder.

The bounded outbound TLS-over-TCP contract and Network.framework adapter have separate [connection](capabilities/connection.md) and [iOS connection](ios/connection.md) guides.

The synchronous app-refresh contract and `BGAppRefreshTask` adapter have separate [background-task](capabilities/background-tasks.md) and [iOS background-task](ios/background-tasks.md) guides.
