# Capability support

[`capability-status.json`](capability-status.json) is the canonical machine-readable and
human-readable support manifest. It has 113 capability rows across 15 families. Each row names a
crate/module, portability class, iOS implementation class, verified platform metadata, parity and
performance status, and native-escape status.

| iOS class | Meaning |
| --- | --- |
| `R` | Portable Rust implementation selected after correctness and performance evidence |
| `M` | Rust semantics/state over a minimal public Apple primitive |
| `B` | Apple/system-owned backend reached through Rust |
| `A` | Apple implementation preferred for performance or hardware reasons |
| `C` | Compiler/build/discovery contract |
| `X` | No supported implementation in this workstream; reason is in the row |

Current counts: 85 rows have `B` support, and all remain partial at the whole-capability
level pending runtime/parity evidence where applicable: two partial UIKit example rows, two partial
reusable UI-control rows, one partial native-escape row, one partial finite-Frame geometry row, one
partial CoreGraphics frame-intersection row, one partial CoreText system-font-metrics row,
three partial B1 sandbox-file and preference rows, and one partial row each for B2 Keychain, B3
request/response model, foreground HTTP, B4 local notifications, B5 current location, B6 plain-text clipboard, B7 outgoing
share, B8 accessibility, B9 acknowledgement alert, B10 packaged resources, B11 external HTTPS URL
handler, B12 local notification response, B13 background transfer, B15 motion, B16 local
authentication, B17 file coordination, B18 informational network-path snapshot, B19 raw-byte
CFData copies, B20 strict Foundation URL values, B21 CoreGraphics frame intersection, B22 finite
CoreMedia media time, B23 CoreText system-font metrics, B24 outbound TLS-over-TCP, B25 app-refresh
scheduling, B26 ImageIO metadata, B27 Photos authorization, B28 UIKit background execution, B29
Contacts authorization, B30 Calendar authorization, B31 HealthKit authorization, B32 Bluetooth
authorization, B33 WebKit navigation, B34 Bluetooth discovery, B35 iCloud Drive identity
presence, B36 camera/microphone authorization status, B37 NFC reader support, B39 one Nearby
Interaction device-capability query, B40 App Tracking Transparency status, B41 Metal
default-device presence, B42 DeviceCheck/App Attest API support, B43 Watch Connectivity session
support, B44 app-visible ExternalAccessory list presence, B45 legacy ReplayKit availability, B46
built-in SoundAnalysis classifier recognition, B47 other-app audio playback status, B48 HDR playback
eligibility, B49 MessageUI mail/text and SharedWithYou status, B50 VideoToolbox hardware-decode support,
B51 Apple Pay capability status, B52 CloudKit account-status snapshot, B53 SafetyKit Crash Detection
availability, B54 ARKit world-tracking support, B55 Game Center local-player status, B56 Core ML
compute-device availability, B57 Vision text-recognition revision support, B58 Speech authorization
status, B59 English Natural Language asset status, B60 legacy StoreKit purchase-ability status, B61 RoomPlan device-support status, B62 default camera video-device status, B63 StoreKit 2 purchase-ability status, B64 SafariServices HTTPS presentation, B65 Accelerate vDSP vector addition, B66 CommonCrypto SHA-256 only, B67 ModelIO extension support only, B68 MPS preferred-device presence only, B69 P-256 verification suitability only, B70 SpriteKit node position only, B71 MediaPlayer authorization status only, B72 CallKit call snapshot only, B73 MapKit geometry only, and B74 ClassKit deep-link marker only, B75 FileProvider registered-domain presence only, B76 ProximityReader device-model support only, B77 extension-point metadata read only, and B78 Sign in with Apple credential-state query only, and B79 Personal VPN profile-status query only; 28 rows are `X` for iOS runtime support. Counts are per capability row, so B3's shared `ios-network` backend appears for both the
request/response model and foreground HTTP. All 28 remaining `X` rows name a specific scope or toolchain gap; no generic missing-facade/backend reason remains. `X` is current
workstream status, not proof that Rust cannot call an Apple API. D90–D99 audits record scoped outcomes for rows 104–113; rows 104–112 remain `X`, while row 113 has B77's runtime metadata partial; see the linked [D90](../../PLAN_CAPABILITIES_MARKETPLACEKIT.md), [D91](../../PLAN_CAPABILITIES_MATTERSUPPORT.md), [D92](../../PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md), [D93](../../PLAN_CAPABILITIES_CARKEY.md), [D94](../../PLAN_CAPABILITIES_PROXIMITYREADER.md), and [D95](../../PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md), [D96](../../PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md), [D97](../../PLAN_CAPABILITIES_WIDGETKIT.md), [D98](../../PLAN_CAPABILITIES_ACTIVITYKIT.md), and [D99](../../PLAN_CAPABILITIES_APP_INTENTS.md) audits. HomeKit is one specific scope gap:
its authorization status is an instance property and first manager use can prompt, so a prompt-free
status-only facade is not claimed. The
AccessorySetupKit is another specific scope gap: the public `ASAccessorySession` surface has no
prompt-free support predicate; session state requires a configured discovery flow, and picker
presentation is user-facing. An OS-version check alone would report API availability, not usable
capability. See row 044 in the [canonical manifest](capability-status.json).
Row 021 is partial (`B`) only for B78's prior-user credential-state query under a conservative Sign in with Apple entitlement requirement. Passkeys, general sign-in readiness, and the original non-entitled scope remain unsupported ([D66 audit and B78 follow-up](../../PLAN_CAPABILITIES_PASSKEYS.md)).
Row 074 ScreenTime/FamilyControls also remains unsupported: its status property is Swift-only and
main-queue-only, no generated Rust binding is available, and Apple does not document query-only
entitlement semantics ([D67 feasibility plan](../../PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md)).
Row 067 Foundation Models remains unsupported because its narrow availability snapshot is Swift-only,
with no public C/Objective-C entry point or generated Rust binding; a supported wrapper or Swift ABI
boundary is not integrated ([D69 feasibility plan](../../PLAN_CAPABILITIES_FOUNDATION_MODELS.md)).
Row 075 DeviceActivity/ManagedSettings remains unsupported even though a public Objective-C getter
is callable in principle: Apple does not define its authorization meaning, prompt behavior, threading,
or query-only entitlement semantics ([D71 feasibility plan](../../PLAN_CAPABILITIES_DEVICE_ACTIVITY.md)).
Row 092 TipKit remains unsupported because `Tip.status` is a Swift-only per-tip eligibility value,
not global readiness or proof of presentation; no C/Objective-C entry point or generated Rust binding
is available ([D73 feasibility plan](../../PLAN_CAPABILITIES_TIPKIT.md)).
Rows 033/036/042 remain unsupported for distinct host/service gates: PushKit's cached VoIP token is
not a readiness query and a real path needs APNs/delegate and call-handling lifecycle; BackgroundAssets
has no global readiness query and depends on host extension/configuration; SensorKit's reader status
requires Apple-approved research entitlement enforced at app launch. See the [PushKit](../../PLAN_CAPABILITIES_PUSHKIT.md),
[BackgroundAssets](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md), and [SensorKit](../../PLAN_CAPABILITIES_SENSORKIT.md)
feasibility plans.
Row 045 Thread remains unsupported: `THClient.isPreferredNetworkAvailableWithCompletion:` (iOS 16.4+)
reports preferred-network availability, needs `com.apple.developer.networking.manage-thread-network-credentials`
and Apple distribution approval, and does not establish local radio or border-router capability.
The Swift-only `MatterAddDeviceRequest.isSupported` query covers Matter setup, not Thread
([D75 feasibility plan](../../PLAN_CAPABILITIES_THREAD.md)).
Row 097 DockKit remains unsupported because its iOS 17+ setting and event APIs are Swift-only, with
no public Objective-C/C declaration, generated Rust binding, or integrated Swift ABI boundary.
`isSystemTrackingEnabled` reports a system setting, not accessory presence or active tracking;
`accessoryStateChanges` needs async lifecycle support ([D78 feasibility plan](../../PLAN_CAPABILITIES_DOCKKIT.md)).
Row 091 WeatherKit remains unsupported as a native Rust package: the SDK API is Swift-only, while REST needs trusted server-signed developer tokens and a separate typed-data/attribution contract ([D79 audit](../../PLAN_CAPABILITIES_WEATHERKIT.md)). Row 095 RealityKit remains unsupported for useful scene work: `ARView` is only a view shell and scene/entity operations are Swift-facing ([D80 audit](../../PLAN_CAPABILITIES_REALITYKIT.md)).
Row 098 NetworkExtension/VPN is partial (`B`) only for B79's read-only caller-app Personal VPN profile-status snapshot after preference load. The host requires `com.apple.developer.networking.vpn.api = ["allow-vpn"]`; this does not cover preference mutation, tunnel control, provider extensions, routes, reachability, or system-wide VPN state ([D82 audit and B79 plan](../../PLAN_CAPABILITIES_NETWORK_EXTENSION.md)).
Row 084 PushToTalk remains unsupported because no standalone support/authorization query exists and useful operation needs entitlement, background mode, microphone consent, APNs channel restoration, and audio lifecycle ([D83 audit](../../PLAN_CAPABILITIES_PUSHTOTALK.md)). Row 085 CarPlay remains unsupported because its session-configuration values describe the connected vehicle, not device availability; useful access requires an approved category entitlement and host scene/session lifecycle ([D84 audit](../../PLAN_CAPABILITIES_CARPLAY.md)). Row 100 ExtensionKit/Foundation remains unsupported: the only status-like API is a Swift-only asynchronous inventory for a host-defined extension point, while iOS host/browser controllers need host UI plus extension process/XPC lifecycle ([D86 audit](../../PLAN_CAPABILITIES_EXTENSIONKIT.md)). Row 102 ContactProvider remains unsupported because `ContactProviderManager.isEnabled` is Swift-only and reports person-enabled state rather than extension health or sync; enabling may prompt and the extension needs its host metadata ([D87 audit](../../PLAN_CAPABILITIES_CONTACTPROVIDER.md)). Row 099 FileProvider now has B75 partial support for registered-domain presence in the caller app; its scoped compile/Clippy/rustdoc/link gates passed, but this does not prove provider enablement, sync, or file access ([D85/B75](../../PLAN_CAPABILITIES_FILEPROVIDER.md)). Row 101 BrowserEngineKit has generated bindings, but no host/process/XPC implementation or Apple entitlement approval; grant validity does not establish general engine support ([D88 audit](../../PLAN_CAPABILITIES_BROWSERENGINEKIT.md)). Row 103 ManagedApp/Distribution has Swift-only configuration and catalog APIs, with no general managed-device status and an entitlement-gated distribution operation ([D89 audit](../../PLAN_CAPABILITIES_MANAGEDAPP.md)).
Row 089 AdAttributionKit/AdServices remains unsupported under the no-Swift rule: the iOS 18
`AppImpression.isSupported` scalar is Swift-only, while AdServices token generation is a network
operation rather than a status check ([D74 feasibility plan](../../PLAN_CAPABILITIES_AD_ATTRIBUTION.md)).
The four D1 portable contracts cover application lifecycle, sandbox files/directories, preferences,
and foreground HTTP values. D7 adds the read-only `framework-resources` contract for exact paths
inside packaged resources. B10 adds an iOS main-bundle backend for exact ordinary files, with an iOS
4.0 API floor; no live read, localization, asset-catalog access, or symlink-containment claim is made.
D8 adds borrowed, syntax-validated RFC 3986 `Uri` and `UriReference` values in `framework-format`
([guide](uri.md)); they do not normalize, percent-decode, or resolve references. B11 separately uses
`Uri` to request an external HTTPS URL handler; it does not guarantee Safari or page load.
B64 separately constructs an HTTPS-only `SFSafariViewController` and exposes a borrowed
`UIViewController` for host-managed presentation and dismissal ([iOS guide](../ios/safari.md)). The
host owns the UIKit lifecycle; construction does not guarantee presentation, a URL request, page
load, or visible content. The SafariServices declaration floor is iOS 9.0, while the scoped Rust
device link floor is iOS 10.0 and the Simulator gate uses iOS 14.0. B65 adds only single-precision
vDSP vector addition on iOS; no portable math contract, parity, or performance claim is made
([iOS guide](../ios/accelerate.md)).
D5 adds a partial portable plain-text clipboard contract; B6 adds an iOS general-pasteboard backend
with documented iOS privacy behavior and item replacement. No live
privacy prompt or paste behavior is claimed. B1 adds iOS sandbox file and
`NSUserDefaults` backends for three rows. B17 adds a separate `IosFileCoordinator` extension for
synchronous Foundation coordination of caller-supplied file URLs ([guide](../ios/file-coordination.md));
it does not add provider, picker, or security-scope lifecycle support or change D1 sandbox paths.
B2 adds a public Keychain generic-password backend for opaque bytes with explicit protection
requirements; no live Keychain test is claimed. B3 adds the Foundation `URLSession` foreground
HTTP backend; no runtime request or Apple parity test is claimed. D10 adds durable GET file-download
values; B13 adds a Foundation background `URLSession` backend using B14 file adoption ([guide](../ios/transfer.md)). It requires app-owned event forwarding and does not claim runtime relaunch or force-quit evidence; an ambiguous commit crash can leave the last durable status `Active`. D3 adds a partial portable local-
notification contract; B4 adds an iOS local-notification backend but not remote push. D4 adds a
partial portable one-shot location contract; B5 adds an iOS Core Location backend for foreground
current location only, with no continuous updates, geofencing, significant-change monitoring, or
background operation. D6 adds a partial portable outgoing-share contract for UTF-8 text and URL
text; B7 adds UIKit `UIActivityViewController` presentation for those items from an explicit
same-window presenter/source-view context, with an iOS 8.0 floor. No live share UI, recipient
delivery, or unforeseen UIKit presentation recovery is claimed. B8 adds synchronous accessibility
metadata setters for a borrowed `UIView`; its iOS 6.0 API floor is declaration-derived, while this
host's SDK/link deployment minimums are iOS 12.0 for device and iOS 14.0 for simulator. No live
VoiceOver behavior is claimed. B9 adds a synchronous one-action `UIAlertController` acknowledgement
alert with an iOS 9.0 API floor ([iOS guide](../ios/presentation.md)). UIKit's presentation method
has no failure callback; no live display or dismissal is claimed. The UIKit slice remains partial: a
Rust-owned app delegate and one window, plus D14's reusable container, label, button, finite Frame,
owned target/action callback, and capability-specific native handles.
The manifest counts thirty-six portable contracts as implemented and twenty-two as partial (58 portable-contract rows total). D9 adds owned
notification-response values in `framework-notifications` ([guide](../notification-responses.md));
B12 adds an opt-in iOS local-response delegate bridge ([guide](../ios/notification-responses.md)).
The app must retain its handle and serialize access to the shared delegate; the callback queue is
unspecified, invalid and remote-push responses are dropped, and no live-delivery claim is made.
D11 adds `framework-motion` ([guide](motion.md)) with a one-shot raw accelerometer contract; B15
adds the iOS backend ([guide](../ios/motion.md)). Device/simulator compile and lint checks do not
claim physical-sensor behavior. D12 adds `framework-auth` ([local-authentication guide](authentication.md))
for one-shot biometric-only or device-owner policy checks; B16 adds the iOS LocalAuthentication
backend ([iOS guide](../ios/authentication.md)). DeviceOwner requires iOS 9.0 and BiometricsOnly
starts at iOS 8.0. No live prompt, biometric-data, or identity-proof claim is made. D13 adds
`framework-data` ([data guide](data.md)) for exact borrowed byte/UTF-8 views, explicit owned copies,
and allocation-transfer conversions; B19 adds explicit raw-byte copies to and from immutable Core
Foundation `CFData` through the [iOS data guide](../ios/data.md), while native UTF-8/string
conversion remains unsupported. D15 adds the
[portable connectivity contract](connectivity.md) for one informational path snapshot; B18 adds
the separate [iOS Network.framework backend](../ios/connectivity.md). It does not preflight or gate
requests, and compile/link checks do not prove live path or cancellation behavior. B20 adds a strict
`NSURL` adapter for D8's absolute `Uri` values ([iOS URL guide](../ios/url.md)); it rejects input
Foundation does not accept without automatic invalid-character encoding, keeps the exact source
text, and has an iOS 17.0 API floor. Its compile/link checks do not prove parser acceptance or
component parity.

D16 adds exact positive-area intersection for finite `Frame` values in the portable UI guide.
B21 adds one CoreGraphics-backed `CGRectIntersection` operation through the [iOS UI guide](../ios/ui.md).
The portable and iOS implementations share edge/touching semantics; CoreGraphics runtime parity
remains untested, so the broad graphics row is still partial. D17 adds finite rational `MediaTime`
values with exact comparison; B22 maps them to CoreMedia `CMTime` by value ([portable media guide](media.md),
[iOS media guide](../ios/media.md)). The adapter has an iOS 4.0 API floor; no media buffers, capture,
playback, or AVFoundation API is included, and the probes do not test runtime behavior.

D18 adds portable finite `FontMetrics` values and a `TextMetricsBackend` contract for system-font
ascent, descent, and leading. B23 queries those metrics through public CoreText C calls ([portable
guide](text-metrics.md), [iOS guide](../ios/text-metrics.md)); the SDK API floor is iOS 3.2. It does
not shape text or measure width, line breaks, paragraphs, custom fonts, or Dynamic Type. The
device/Simulator Release probes check signatures, layout, and imports but do not call CoreText or
establish `UILabel` parity.

D20/B25 adds one synchronous `BGAppRefreshTask` path using public BackgroundTasks APIs. Task launch
time is OS-controlled; host task registration and `BGTaskSchedulerPermittedIdentifiers` /
`UIBackgroundModes` entries remain app-owned. The compile/link/import gates do not establish task
delivery, expiry handling, relaunch, or performance. It does not implement `BGProcessingTask` or
replace B13 background URLSession downloads; D23/B28 separately covers one UIKit execution lease.

D19 adds `framework-connection` for one bounded outbound TLS-over-TCP byte stream; B24 uses public
Network.framework C APIs ([portable guide](connection.md), [iOS guide](../ios/connection.md)).
Send/receive chunks are limited to 1 MiB and operations are serialized; it does not add listeners,
UDP, Bonjour, HTTP, or WebKit. Local-network privacy metadata remains host-owned when applicable,
and compile/link/import evidence is not a live peer or TLS test.

D21 adds `framework-image` for image-zero encoded dimensions and total source count; B26 reads only
those metadata values through ImageIO ([portable guide](image-metadata.md), [iOS guide](../ios/image-metadata.md)). It does not request raster output or claim general CoreImage/ImageIO support

D22 adds `framework-photos` for explicit read/write authorization status and request; B27 uses
PhotoKit's read/write access level and preserves `Limited` separately ([portable guide](photos.md),
[iOS guide](../ios/photos.md)). It does not enumerate assets, request image data, edit assets, or
present a picker; the host app owns `NSPhotoLibraryUsageDescription`

D23/B28 adds one UIKit background-execution lease with explicit end and a cooperative expiry
signal ([portable guide](background-execution.md), [iOS guide](../ios/background-execution.md)).
It does not promise extra runtime, future launch, continued execution after expiry, or work
completion; app extensions are unsupported.

D24/B29 adds Contacts authorization status and explicit request only ([portable guide](contacts.md),
[iOS guide](../ios/contacts.md)). `Limited` remains distinct from full access; the host app owns
`NSContactsUsageDescription`. Enumeration, fetch, edits, picker UI, and live consent/data behavior
remain out of scope.

D25/B30 adds EventKit Calendar event-authorization status and explicit full-access request only
([portable guide](calendar.md), [iOS guide](../ios/calendar.md)). `WriteOnly` remains distinct from
`FullAccess`; the host app owns `NSCalendarsFullAccessUsageDescription`. The backend targets iOS
17.0 or later and does not read or write events, request reminders, or present calendar UI.

D26/B31 adds borrowed HealthKit type values and an explicit read/share authorization request only
([portable guide](health-authorization.md), [iOS guide](../ios/health-authorization.md)). The backend
checks HealthKit availability before calls and does not infer access from request completion or
expose samples. The host app owns the HealthKit entitlement and conditional usage descriptions.

D27/B32 adds a non-prompting Bluetooth authorization snapshot through the public
`CBManager.authorization` class property. The adapter targets iOS 13.1 or later and does not create
a manager or implement radio state. D29/B34 adds explicit unfiltered foreground central discovery
with copied peer UUID/RSSI values and a bounded 32-event queue. Starting a scan may prompt and
returns before readiness/results; there is no background-delivery guarantee, connection,
peripheral/advertising operation, or radio control. Discovery identity has an iOS 8.0 floor; static
authorization remains iOS 13.1. Apps linked on or after iOS 13 need
`NSBluetoothAlwaysUsageDescription`; apps whose deployment target predates iOS 13 need both that and
`NSBluetoothPeripheralUsageDescription`.

D28/B33 adds a platform-exclusive borrowed HTTPS URL and navigation contract with a typed iOS
`WKWebView` adapter. The local typed objc2 declarations fill a generated iOS binding gap; the adapter
supports attach, back/forward state and actions, reload, and stop only. It does not provide a
JavaScript bridge, arbitrary file/HTML/data loads, a subresource firewall, browser parity, or a live
page-load guarantee. See [portable web](web.md) and [iOS web](../ios/web.md) guides.

D30/B35 adds an iCloud Drive Documents identity-presence snapshot only. It maps a nullable
`NSFileManager.ubiquityIdentityToken` to present/absent and never returns or retains the token. Nil
has multiple causes; presence does not prove container access, sync, or CloudKit account status. See
[portable iCloud identity](icloud-drive-identity.md) and [iOS iCloud identity](../ios/icloud-drive-identity.md) guides.

D31/B36 queries camera and microphone authorization status through AVFoundation only. It does not
request permission, enumerate devices, create capture/audio sessions, or access media samples.
`NSCameraUsageDescription` and `NSMicrophoneUsageDescription` apply before a host requests access or
attempts capture; this query does neither. See [portable media authorization](media-authorization.md)
and [iOS media authorization](../ios/media-authorization.md) guides.

D32/B37 queries `NFCReaderSession.readingAvailable` only. It reports reader support, not permission,
session readiness, tag discovery, or tag I/O; no NFC session is created. The iOS 11.0 API floor and
conservative `NFCReaderUsageDescription` caveat are documented in the [NFC](nfc.md) and
[iOS NFC](../ios/nfc.md) guides.

D34/B39 reports only Nearby Interaction's `supportsPreciseDistanceMeasurement` device capability
through `NISession.deviceCapabilities` on iOS 16.0+. It does not report permission, peer
compatibility, session readiness, or operation success; no session or ranging operation is run. See
[Nearby Interaction](nearby-interaction.md) and [iOS Nearby Interaction](../ios/nearby-interaction.md).

D35/B40 reports only the calling app's App Tracking Transparency authorization status through
`ATTrackingManager.trackingAuthorizationStatus` on iOS 14.0+. It does not request authorization,
access IDFA, or track. `NSUserTrackingUsageDescription` remains required host configuration for ATT
API use. See [privacy authorization](privacy-authorization.md) and
[iOS tracking authorization](../ios/tracking-authorization.md).

D36/B41 reports only whether `MTLCreateSystemDefaultDevice` returns a device object on iOS 8.0+.
It drops the temporary retained object and submits no work; no MetalKit, rendering, compute, feature,
or performance support is claimed. See [Metal](metal.md) and [iOS Metal](../ios/metal.md).

D37/B42 reports only the DeviceCheck and App Attest `isSupported` values, with iOS floors of 11.0
and 14.0. It does not create keys or tokens, attest or assert, contact a service, or establish trust.
D38/B43 reports only `WCSession.isSupported()` on iOS 9.0+; it does not retrieve or activate a
session, inspect pairing, or communicate. D39/B44 reports only whether the current
`EAAccessoryManager.connectedAccessories` list is empty on iOS 3.0+; it does not identify hardware,
open a session, or communicate. D40/B45 reports only the legacy `RPScreenRecorder.isAvailable`
value on iOS 9.0+; Apple currently marks it deprecated and recommends ScreenCaptureKit, which this
slice does not implement. See the [DeviceCheck guide](device-integrity.md), [Watch Connectivity
guide](watch-connectivity.md), [ExternalAccessory guide](accessory.md), [ReplayKit guide](replaykit.md),
and their [iOS guides](../ios/device-integrity.md), [Watch Connectivity](../ios/watch-connectivity.md),
[ExternalAccessory](../ios/external-accessory.md), and [ReplayKit](../ios/replaykit.md).

D41/B46 reports only whether the built-in SoundAnalysis version 1 classifier request is recognized
on iOS 15.0+. It creates no analyzer, supplies no audio, requests no microphone access, and does not
cover ShazamKit. See [SoundAnalysis](sound-analysis.md) and [iOS SoundAnalysis](../ios/sound-analysis.md).

D42/B47 reports only whether any other app is playing audio at query time, through
`AVAudioSession.isOtherAudioPlaying` on iOS 6.0+. It includes ambient-category audio, but does not
identify the source, report this app's playback, or expose Now Playing metadata or media control.
See [Other Audio](other-audio.md) and [iOS Other Audio](../ios/other-audio.md). D43/B48 adds a portable
HDR-eligibility snapshot through `AVPlayer.eligibleForHDRPlayback` from iOS 13.4; it does not inspect
a specific asset or start playback ([portable guide](playback.md), [iOS guide](../ios/playback.md)).
D44/B49 adds platform-exclusive mail/text availability and SharedWithYou software-support status;
it does not compose or send messages, inspect accounts, or access collaboration data ([MessageUI](../ios/message-ui-support.md),
[SharedWithYou](../ios/shared-with-you-support.md)). D45/B50 asks VideoToolbox only whether the system reports hardware decode support for a caller
codec FourCC; it does not encode, decode frames, inspect assets, or reserve decoder resources
([portable guide](video-codec.md), [iOS guide](../ios/videotoolbox.md)). D46/B51 reads only PassKit's general Apple Pay capability
predicate on iOS 10.0+; it does not inspect cards, merchant networks, or process a payment
([guide](apple-pay.md), [iOS guide](../ios/apple-pay-availability.md)).

D47/B52 adds one CloudKit account-status snapshot from the app's default container; it does not read
CloudKit data, observe account changes, or prove container/database access
([portable guide](cloudkit-account-status.md), [iOS guide](../ios/cloudkit-account-status.md)).

D48/B53 reads only SafetyKit's Crash Detection device-support bit; it does not claim event
entitlement or authorization, receive crash events, or provide emergency response. The SDK header
states a class entitlement, but the getter-specific prerequisite remains unspecified
([iOS guide](../ios/safetykit.md)).

D49/B54 adds one `WorldTrackingSupport` value and queries ARKit configuration support only; it
does not create a session, access the camera, request camera permission, or claim active tracking
([portable guide](arkit.md), [iOS guide](../ios/arkit.md)).

D50/B55 adds an allocation-free portable local-player authentication-status contract and one
non-prompting Game Center Boolean read; it does not initialize authentication, expose identity or
game data, or observe changes. A configured app requires the signed Game Center entitlement
([portable guide](gamekit-status.md), [iOS guide](../ios/game-center-status.md)).

D51/B56 checks only whether the iOS Core ML compute-device list is nonempty; it does not load a
model or run inference, and it makes no claim that a particular model can run
([iOS guide](../ios/core-ml-device-status.md)).

D52/B57 checks only whether a caller-supplied text-recognition revision appears in Vision's
supported revision set. It creates no request, reads no image, and claims no recognition or model
readiness result ([portable guide](vision.md), [iOS guide](../ios/vision.md)).

D53/B58 reads only the app's saved Speech authorization status. It does not request permission,
create a recognizer, accept audio, or start recognition; authorization does not establish service
availability or recognition success ([iOS guide](../ios/speech-status.md)).

D54/B59 checks only the on-device asset state for Apple’s built-in English contextual model. It does
not load the model, accept text, compute vectors, request assets, or guarantee a later model
operation ([iOS guide](../ios/natural-language-status.md)).

D55/B60 exposes only the deprecated StoreKit 1 `SKPaymentQueue::canMakePayments` bit. It does not
create a queue, inspect products or accounts, present payment UI, or process transactions. D58/B63
adds only the StoreKit 2 `AppStore.canMakePayments` status; product, transaction, and portable
commerce contracts remain out of scope ([D55 iOS guide](../ios/storekit-status.md),
[D58 iOS guide](../ios/storekit2-status.md)).

D56/B61 exposes a portable `RoomPlanDeviceSupport` value and the iOS 16.0+
`RoomCaptureSession.isSupported` predicate through a compiler-verified `swiftcall` thunk. It does not
create a session, access camera/LiDAR frames, request permission, or claim scan success ([portable
guide](roomplan.md), [iOS guide](../ios/roomplan-status.md)).

D57/B62 adds an iOS 4.0+ default-video-device presence query through
`AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`. It adds no portable camera contract,
authorization query, capture session, media access, or UI ([iOS guide](../ios/camera-device-status.md)).

D58/B63 exposes the iOS 15.0+ `AppStore.canMakePayments` Boolean through a weak-import
`swiftcall` thunk. The absent-symbol fallback is `false`; the query adds no product, account,
entitlement, purchase UI, or transaction behavior ([iOS guide](../ios/storekit2-status.md)).

D59/B65 exposes only iOS 4.0+ single-precision `vDSP_vadd` vector addition through equal-length
borrowed `f32` slices. It adds no portable math contract, other Accelerate operation, or performance
claim ([iOS guide](../ios/accelerate.md)).

B66 adds only the iOS 2.0+ CommonCrypto `CC_SHA256` operation for owned digest bytes. The portable
crypto facade and system/hardware key escape remain unimplemented; no replacement, parity, or
performance claim is made.

B67 adds only the iOS ModelIO `MDLAsset.canImportFileExtension` query. It reports extension-level
support only; it does not load or parse a file, render an asset, or claim GPU support ([iOS guide](../ios/modelio-status.md)).

B68 adds only whether `MPSGetPreferredDevice` returns a device with default options. It does not
submit GPU work or establish support for any MPS operation or workload ([iOS guide](../ios/mps-status.md)).

B69 adds one borrowed P-256 public-key import and ECDSA/SHA-256 message-verification suitability
query. It does not verify a signature, use or persist a private key, or access the Secure Enclave
([portable guide](key-support.md), [iOS guide](../ios/key-support.md)).

B70 adds a portable finite parent-local position contract and a detached iOS `SKNode` create/get/set
facade for `position` only. The API floor is iOS 7.0; the device/Simulator link probes use 12.0/14.0
SDK validation minimums. It does not add SceneKit, scenes, rendering, hierarchy, animation, or physics
([portable guide](spritekit-node-position.md), [iOS guide](../ios/spritekit-node-position.md)).

B71 reads only the point-in-time MediaPlayer library authorization status on iOS 9.3+; it does not
request access, read library items, or contact Apple Music services ([iOS guide](../ios/media-library-status.md)).

B72 reads `CXCallObserver.calls` once and copies a count plus aggregate outgoing, connected, on-hold,
and ended flags. The synchronous initial read may block; this does not expose call objects, UUIDs,
caller data, callbacks, call control, providers, PushKit, or audio ([iOS guide](../ios/call-observer.md)).

B73 adds finite caller-supplied map geometry: coordinate/map-point conversion and distance. It does
not add map UI, user location, permissions, network service, search, or directions
([portable guide](mapkit.md), [iOS guide](../ios/mapkit.md)).

B74 reads only `NSUserActivity.isClassKitDeepLink` from a caller-owned activity on iOS 11.3+. It
does not access `CLSDataStore`, assignment content, context identifiers, or user identity. The
getter has no documented entitlement; Schoolwork data sharing is a separate host capability
([ClassKit plan](../../PLAN_CAPABILITIES_CLASSKIT.md)).

| Family | Rows | iOS class count |
| --- | ---: | --- |
| Core app/UI | 9 | `B`: 9 (2 partial UIKit example rows; 2 partial reusable UI-control rows; 1 partial native-escape row; 1 partial clipboard row; 1 partial share row; 1 partial accessibility row; 1 partial acknowledgement-alert row); `X`: 0 |
| Files/data/preferences | 7 | `B`: 7 partial; `X`: 0 |
| Security/auth | 7 | `B`: 7 partial; `X`: 0 |
| Networking/web | 7 | `B`: 7 partial, including external HTTPS handling, WebKit navigation, and host-presented SafariServices construction; `X`: 0 |
| Notifications/background | 6 | `B`: 4 partial; `X`: 2 |
| Sensors/connectivity | 9 | `B`: 6 partial; `X`: 3 |
| Camera/audio/media | 8 | `B`: 8 partial including VideoToolbox hardware-decode support; `X`: 0 |
| Graphics/GPU | 9 | `B`: 9 partial finite-Frame geometry, CoreGraphics frame-intersection, CoreText system-font-metrics, ImageIO metadata, Metal device-presence, Accelerate vDSP vector-add, ModelIO extension-support, MPS preferred-device, and SpriteKit node-position rows; `X`: 0 |
| ML/vision/language | 5 | `B`: 4 partial Core ML compute-device, Vision revision-status, Speech authorization-status, and Natural Language English-model asset rows; `X`: 1 |
| Personal data/system stores | 8 | `B`: 5 partial Photos, Contacts, Calendar, HealthKit authorization, and SafetyKit availability rows; `X`: 3 |
| Cloud/accounts/communication | 10 | `B`: 8 partial including CloudKit, CallKit, ClassKit marker, Game Center, and MessageUI/SharedWithYou status; `X`: 2 |
| Commerce/services | 7 | `B`: 4 partial Apple Pay, legacy StoreKit, StoreKit 2 purchase-ability, and MediaPlayer library-authorization status; `X`: 3 |
| Maps/AR/spatial | 5 | `B`: 3 partial MapKit geometry, ARKit world-tracking, and RoomPlan device-support rows; `X`: 2 |
| Extension/entitlement capabilities | 13 | `B`: 4 partial; `X`: 9 |
| Compiler/build-host capabilities | 3 | `X`: 3 |

For unverified platform metadata the manifest uses `null`, not an inferred empty requirement. A
verified empty list means the row's source documents no required item for that field. The B UI
facts come from [`docs/ios/runtime.md`](../ios/runtime.md) and the `ios-minimal` example; no iOS
minimum version is stated there, so the manifest leaves it unknown. No entitlement, permission, or
framework requirement is inferred for an `X` row. A `true` native-escape value names the handle
documented for that row: UIKit example handles or a capability-specific borrowed native object.
The C7 [App Intents audit](../swift-abi/APP_INTENTS_STAGE0.md) found no stable Rust/C metadata input
or processor API on Xcode 26.6; the Stage 1 runtime path remains unsupported, with no fake capability
API. Re-audit on the Xcode 27.x baseline.

See [the D1 API guide](app-data.md) for the four portable crate contracts, [the secure-storage
guide](secure-storage.md) for D2's opaque-byte contract, [the data guide](data.md) for D13's
borrow/copy/ownership-transfer behavior, [the connectivity guide](connectivity.md) for D15's
advisory path status, [the UI guide](ui.md) for D14's finite frame and native label/button
slice, and [the font-metrics guide](text-metrics.md) for D18's limited portable values. These guides detail ownership, copy, atomicity, async/cancellation,
errors, and runtime limits. D3–D81 are tracked in separate named subplans; D15's implemented
contract and backend are counted as partial support in row 028, while B21's finite CoreGraphics
operation is counted in row 055, B22's finite time value is a partial slice of row 049, B23's
system-font metrics are a partial slice of row 056, B26 is a partial slice of row 057, B27 is a
partial slice of row 068, B28 is a partial slice of row 035, B29 is a partial slice of row 069,
and B30 is a partial slice of row 070, B31 is a partial slice of row 071, B32 and B34 are partial
slices of row 039, B33 is a partial slice of row 029, B35 is a partial slice of row 077, B36 is
a partial slice of rows 046 and 047, B37 is a partial slice of row 040, B39 is a partial slice
of row 041, B40 is a partial slice of row 023, B41 is a partial slice of row 058, B42 of row 022,
B43 of row 078, B44 of row 043, B45 of row 052, B46 of row 053, B47 of row 051, B48 of row 048, B49 of rows 081–082, B50 of row 050, B51 of row 088, B52 of row 076, B53 of row 073, B54 of row 094, B55 of row 079, B56 of row 063, B57 of row 064, B58 of row 066, B59 of row 065, B60 of row 087, B61 of row 096, B62 of row 046, B63 of row 086, B64 of row 030, B65 of row 060, B66 of row 018, B67 of row 061, B68 of row 059, B69 of row 019, B71 of row 090, B72 of row 083, B73 of row 093, and B74 of row 080. Workstream D remains incomplete; these bounded slices do not claim family-wide parity.
The D3 [local-notification guide](../notifications.md) describes the portable scheduling contract;
the B4 [iOS guide](../ios/notifications.md) describes the local-only native backend and its runtime
limits. The D4 [location guide](location.md) describes the one-shot portable current-location
contract; the B5 [iOS guide](../ios/location.md) documents its Core Location backend and limits.
The D5 [sharing guide](sharing.md) describes the plain-text clipboard contract; the B6
[iOS guide](../ios/sharing.md) documents the general-pasteboard backend and native privacy limits.
The D6 [share guide](share.md) describes outgoing text and URL-text values; the B7 [iOS
guide](../ios/sharing.md) documents UIKit presentation context, lifecycle, result, and evidence
limits.
The B8 [accessibility guide](../ios/accessibility.md) documents the borrowed-view setters, trait
replacement semantics, API floor, and absence of live VoiceOver evidence.
The D14 [portable UI guide](ui.md) documents finite local-coordinate frames and owned button
callbacks; the [iOS UI guide](../ios/ui.md) documents UIKit control ownership, main-thread rules,
borrowed native handles, and unverified live behavior. Four partial matrix rows record its bounded
view/control, target/action, native-handle, and finite-Frame surfaces; a general geometry library or
cross-capability escape facade is not provided. Window, controller, scene, navigation, general
layout, and accessibility systems remain outside its scope.

The optional foreign-language Keychain surface is documented in the
[secure-storage C ABI guide](../bindings/secure-storage.md); it does not change the Rust-native
call path or the iOS capability classification.
