# PLAN_CAPABILITIES.md — Workstream D: Portable Capability Facades and V1 iOS Coverage

## Objective

Build the high-level developer-facing Rust capability API and assemble the complete V1 iOS backend from Workstreams B/C/E.

The portable API models semantic capabilities, not Apple class names.

## Dependencies

Requires Foundation contracts from A.
Consumes iOS native implementations from B, Swift residual implementations from C, and benchmark-selected replacements from E.

## Execution decomposition

Workstream D is too broad for one bounded executor. Start with `PLAN_CAPABILITIES_APP_DATA.md` (D1), which owns the application lifecycle, files, preferences, foreground HTTP facades, and the first complete support manifest. `PLAN_CAPABILITIES_SECURE_STORAGE.md` (D2) owns secure storage. `PLAN_CAPABILITIES_NOTIFICATIONS.md` (D3) owns the bounded local-notification contract. `PLAN_CAPABILITIES_LOCATION.md` (D4) owns a one-shot portable location contract. `PLAN_CAPABILITIES_CLIPBOARD.md` (D5) owns a portable plain-text clipboard contract in the `framework-sharing` crate; it does not include share-sheet presentation. `PLAN_CAPABILITIES_SHARE.md` (D6) owns a small portable outgoing-share contract in that crate; native presentation remains a separate B workstream. `PLAN_CAPABILITIES_RESOURCES.md` (D7) owns read-only packaged-resource lookup, separate from writable sandbox files. `PLAN_CAPABILITIES_URI.md` (D8) owns borrowed RFC 3986 URI and URI-reference values, with no normalization or resolution. `PLAN_CAPABILITIES_NOTIFICATION_RESPONSES.md` (D9) owns additive portable values for local-notification interaction responses; B12 implements a separate iOS response bridge. `PLAN_CAPABILITIES_TRANSFER.md` (D10) owns the portable durable HTTP download contract; B13 implements its separate iOS backend. `PLAN_CAPABILITIES_MOTION.md` (D11) owns the one-shot accelerometer contract and B15 adds its separate iOS Core Motion backend. `PLAN_CAPABILITIES_AUTHENTICATION.md` (D12) owns one-shot local user-presence checks; B16 implements its separate iOS backend. `PLAN_CAPABILITIES_DATA.md` (D13) owns borrowed byte/UTF-8 views and explicit copy or allocation-transfer conversions in `framework-data`; B19 adds a separate raw-byte CFData adapter, not native string conversion. `PLAN_IOS_UI.md` (D14) adds the small portable `framework-ui` view/label/button contract and the separate UIKit `ios-ui` backend; it does not add window, scene, navigation, layout, or accessibility systems. `PLAN_CAPABILITIES_CONNECTIVITY.md` (D15) owns one informational network-path snapshot; B18 implements it through public Network.framework C APIs in `PLAN_IOS_CONNECTIVITY.md`. The snapshot is not a request preflight and does not claim endpoint reachability. `PLAN_CAPABILITIES_GEOMETRY.md` (D16) adds exact finite-Frame intersection semantics and B21 exposes the CoreGraphics adapter. `PLAN_CAPABILITIES_MEDIA.md` (D17) adds finite rational media time and B22 exposes the CoreMedia `CMTime` value adapter. `PLAN_CAPABILITIES_TEXT_METRICS.md` (D18) adds portable system-font metrics; B23 exposes one CoreText metrics query without text shaping or layout. `PLAN_CAPABILITIES_CONNECTION.md` (D19) adds bounded outbound TLS-over-TCP byte streams in `framework-connection`; B24 is the separate Network.framework C backend. `PLAN_CAPABILITIES_BACKGROUND_TASKS.md` (D20) adds a synchronous app-refresh request/callback contract; B25 is the separate `BGAppRefreshTask` backend. `PLAN_CAPABILITIES_IMAGE_METADATA.md` (D21) owns metadata-only image dimensions and source count; B26 is the separate ImageIO adapter in `PLAN_IOS_IMAGE_METADATA.md`. `PLAN_CAPABILITIES_PHOTOS.md` (D22) owns read/write Photos authorization status and request only; B27 is the separate PhotoKit backend in `PLAN_IOS_PHOTOS.md`. `PLAN_CAPABILITIES_BACKGROUND_EXECUTION.md` (D23) owns one static begin/expiry/end lease contract; B28 is the separate UIKit adapter in `PLAN_IOS_BACKGROUND_EXECUTION.md`. `PLAN_CAPABILITIES_CONTACTS.md` (D24) owns Contacts authorization status and request only; B29 is the separate iOS Contacts adapter in `PLAN_IOS_CONTACTS.md`. `PLAN_CAPABILITIES_CALENDAR.md` (D25) owns Calendar event-authorization status and a full-access request only; B30 is the separate EventKit adapter in `PLAN_IOS_CALENDAR.md`. `PLAN_CAPABILITIES_HEALTH_AUTHORIZATION.md` (D26) owns HealthKit availability and explicit type-scoped read/share authorization requests, without exposing grant status or health data; B31 is the separate public HealthKit adapter. `PLAN_CAPABILITIES_BLUETOOTH.md` (D27) owns the portable Bluetooth authorization snapshot; B32 implements the non-prompting `CBManager.authorization` query. `PLAN_CAPABILITIES_WEB.md` (D28) owns a platform-exclusive HTTPS view/navigation contract; B33 is the bounded iOS `WKWebView` adapter. D29 adds bounded central discovery through `PLAN_IOS_BLUETOOTH_DISCOVERY.md`; B34 is the iOS backend. `PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md` (D30) owns an iCloud Drive identity-presence snapshot; B35 is its Foundation backend. Further capability groups require separate named subplans and executors; they must not overlap D1-owned crates or manifest edits.

D31 (`PLAN_CAPABILITIES_MEDIA_AUTHORIZATION.md`) owns status-only Camera and Microphone authorization values; B36 is the separate AVFoundation query. D32 (`PLAN_CAPABILITIES_NFC.md`) owns the NFC reader-support snapshot; B37 queries Core NFC without creating a session. D33 is a HomeKit status-only feasibility audit; it found no prompt-free manager authorization query. B339 adds only a host-supplied per-accessory `HMAccessory.supportsIdentify` snapshot and B394 reads only the retained `HMAccessory.profiles` array length, so row 072 is partial, not general HomeKit support or profile readiness. D34 (`PLAN_CAPABILITIES_NEARBY_INTERACTION.md`) owns one precise-distance capability value; B39 is the separate non-prompting iOS 16+ query. D35 (`PLAN_CAPABILITIES_PRIVACY_AUTHORIZATION.md`) owns the App Tracking Transparency status-only contract; B40 is the separate iOS 14+ query and does not claim general privacy authorization. D36 (`PLAN_CAPABILITIES_METAL.md`) owns scalar default-device presence; B41 is the separate iOS `MTLCreateSystemDefaultDevice` query and does not claim GPU work or performance.
D37 (`PLAN_CAPABILITIES_DEVICE_INTEGRITY.md`) owns transient DeviceCheck/App Attest support booleans; B42 queries only the public `isSupported` properties. D38 (`PLAN_CAPABILITIES_WATCH_CONNECTIVITY.md`) owns whether a platform can provide a Watch Connectivity session object; B43 calls only `WCSession.isSupported()`. D39 (`PLAN_CAPABILITIES_ACCESSORY.md`) is the app-visible connected-accessory list presence snapshot; B44 counts the public ExternalAccessory list without session or communication. D40 (`PLAN_CAPABILITIES_REPLAYKIT.md`) adds a legacy ReplayKit availability Boolean in `framework-media`; B45 reads only `RPScreenRecorder.isAvailable` and does not start capture or recording. D41 (`PLAN_CAPABILITIES_SOUND_ANALYSIS.md`) adds a built-in classifier-recognition value in `framework-media`; B46 creates the iOS 15.0+ request and checks known labels without supplying audio or accessing a microphone. D42 (`PLAN_CAPABILITIES_OTHER_AUDIO.md`) adds a point-in-time other-audio Boolean; B47 reads only `AVAudioSession.isOtherAudioPlaying` and does not expose Now Playing metadata or media control.

D11 is implemented as [framework-motion](crates/framework-motion) and documented in [the motion guide](docs/capabilities/motion.md); eight deterministic portable contract tests pass, with exact check evidence and fake-backend limits recorded in [the D11 plan](PLAN_CAPABILITIES_MOTION.md). B15 adds a separate [iOS Core Motion backend](docs/ios/motion.md) for one raw device-axis accelerometer sample per request; no other motion sensors, fused values, streams, or sample-rate policy are included. Device/simulator checks are compile/lint evidence only; runtime sensor behavior is not claimed.

D12 adds [framework-auth](crates/framework-auth) and a [local-authentication guide](docs/capabilities/authentication.md) for explicit one-shot biometric-only or device-owner policy checks. B16 implements the separate [iOS LocalAuthentication backend](docs/ios/authentication.md), with device/simulator compile and strict-Clippy checks plus a link/import probe. Success is only the platform policy result at that time; no identity proof or reusable credential is claimed.

## Write scope

- `crates/framework-app/**`
- all `crates/framework-*` capability crates listed in PLAN.md
- capability-level docs/rustdoc/examples
- portable capability tests
- iOS extension modules that expose native handles only through platform-specific namespaces

Do not own low-level Objective-C/Swift ABI machinery.

## Common API design

Every capability crate should expose:
- a small semantic Rust API;
- stable error mapping;
- platform availability/query where behavior differs;
- async as Rust `Future`/operation adapters without mandatory executor;
- cancellation where the OS operation supports it;
- iOS-specific extension/native handle under an explicit namespace;
- no hidden global initialization.

Prefer semantic constructors/operations over one-to-one class mirrors.

Examples of intended style:

```rust
let bytes = Files::read(path).await?;
Preferences::set("key", value)?;
SecureStorage::set("token", bytes).await?;
let location = Location::current(options).await?;
let products = Store::products(ids).await?;
```

Exact names may be refined during API review, but should stay high-level and allocation/cost transparent.

## V1 capability matrix

The following families must be represented in the V1 API surface. A family may be a thin Apple-backed Rust API rather than a reimplementation.

### Core app/UI
- application lifecycle
- windows/screens
- basic native views/controllers/controls
- events/target-action/delegates
- native presentation
- pasteboard/clipboard
- share/system composition
- accessibility extensions
- native escape handles

### Files/data/preferences
- sandbox files/directories
- semantic app directories
- preferences
- bundle/resources
- file coordination/provider-aware operations as platform extensions
- URL/URI value handling
- data/byte/string conversion with explicit copy behavior

### Security/auth
- Keychain secure storage
- software crypto facade with system/hardware key escape
- SecKey/Secure Enclave
- biometrics
- passkeys/Sign in with Apple
- App Attest/DeviceCheck
- privacy authorization status

### Networking/web
- request/response model
- foreground HTTP
- background transfer
- low-level network connection/listener where appropriate
- reachability semantics only via current supported APIs
- WebKit
- Safari/system browser presentation

### Notifications/background
- local/push notification client APIs
- notification response/delegate
- PushKit
- BackgroundTasks
- UIKit background execution
- BackgroundAssets where applicable

### Sensors/connectivity
- location/geofencing/significant-change where supported
- motion
- Bluetooth central/peripheral surfaces required by common apps
- NFC session/tag operations
- Nearby Interaction
- SensorKit
- ExternalAccessory
- AccessorySetupKit
- Thread network/system pieces

### Camera/audio/media
- camera capture/session/device
- microphone/audio engine/I/O
- playback
- media time/buffer wrappers
- video encode/decode through VideoToolbox
- Now Playing/media control
- ReplayKit
- ShazamKit/SoundAnalysis

### Graphics/GPU
- geometry/value types where portable
- CoreGraphics/QuartzCore platform operations
- text layout platform extension
- CoreImage/ImageIO
- Metal/MetalKit
- MPS/MPSGraph
- Accelerate access
- ModelIO
- SceneKit/SpriteKit platform extension

### ML/vision/language
- Core ML generic inference
- Vision
- NaturalLanguage system model features
- Speech
- Foundation Models residual through C where available

### Personal data/system stores
- Photos
- Contacts
- EventKit
- HealthKit
- HomeKit
- SafetyKit
- ScreenTime/FamilyControls base authorization
- DeviceActivity/ManagedSettings where residual support exists

### Cloud/accounts/communication
- CloudKit
- iCloud-related state
- WatchConnectivity
- GameKit
- ClassKit
- MessageUI
- SharedWithYou
- CallKit
- PushToTalk
- CarPlay

### Commerce/services
- StoreKit 2 via C
- legacy StoreKit only where compatibility requires it
- Apple Pay/PassKit
- AdAttributionKit/AdServices as applicable
- Apple Music using REST/MediaPlayer first, Swift residual only where required
- WeatherKit through REST/native HTTP where appropriate
- TipKit only if system TipKit behavior is specifically requested; otherwise framework-owned tip logic may be portable

### Maps/AR/spatial
- MapKit
- ARKit
- RealityKit native/Swift residual coverage that is realistically supportable
- RoomPlan
- DockKit

### Extension/entitlement capabilities
Expose portable or iOS-exclusive facades/documented shells for:
- NetworkExtension/VPN
- FileProvider
- ExtensionKit/Foundation where feasible
- BrowserEngineKit for entitled apps
- ContactProvider
- ManagedApp/Distribution
- MarketplaceKit
- MatterSupport
- SecureElementCredential
- CarKey
- ProximityReader
- LockedCameraCapture

Entitlement-limited APIs must remain explicit iOS capabilities and must not be claimed as generally usable without entitlement.

### Compiler/build-host capabilities
- App Intents: C7 Stage 0 found no documented stable Rust/C metadata input or processor API on Xcode 26.6; Stage 1 is unsupported on the audited toolchain. Expose no fake runtime implementation and re-audit on Xcode 27.x; see [the C7 report](docs/swift-abi/APP_INTENTS_STAGE0.md).
- WidgetKit: support management/data logic available through proven interfaces; do not implement a SwiftUI clone merely to claim full rendering support.
- ActivityKit: support if Layer-2 ABI work is proven.
- extension bundle metadata helpers: only through supported Xcode/public mechanisms.

## Status

D1–D32, D34–D43, D45, D47, D49, D50, D52, D56, D63, and D77 now have scoped portable contracts and guides: app data, secure storage, local
notifications, one-shot location, clipboard, outgoing share, packaged resources, URI values,
notification responses, durable downloads, one-shot accelerometer samples, local authentication,
byte/string views, camera/microphone authorization-status values, bounded UIKit controls/frames with finite Frame intersection, an informational
one-shot network-path snapshot, finite rational media-time values, system-font metrics, bounded outbound TLS-over-TCP byte streams, a synchronous app-refresh contract, image metadata, Photos read/write authorization, a bounded UIKit background-execution lease, Contacts authorization status/request, Calendar authorization status/request, HealthKit authorization-request values, Bluetooth authorization and discovery, an HTTPS web-view/navigation contract, NFC reader support, Nearby Interaction precise-distance capability, iCloud Drive identity presence, App Tracking Transparency status, app-visible ExternalAccessory presence, legacy ReplayKit availability, built-in SoundAnalysis classifier recognition, a point-in-time other-audio snapshot, HDR playback eligibility, VideoToolbox FourCC/hardware-decode status, a partial ARKit world-tracking support value, a local-player Game Center authentication-status contract, and a RoomPlan device-support value. D44 adds platform-exclusive MessageUI and SharedWithYou status APIs without portable contracts; D46 adds one platform-exclusive Apple Pay capability query; D47 adds an owned CloudKit account-status snapshot; D48 adds a platform-exclusive SafetyKit device-support predicate; D55 adds a platform-exclusive legacy StoreKit purchase-status query; D56 adds a portable RoomPlan device-support value with a Swift ABI-backed iOS query; D57/B62 adds a default-video-device presence query to the existing camera row without a new portable contract; D58/B63 adds a StoreKit 2 purchase-ability status query without products or transactions; B64 adds a host-presented SafariServices `SFSafariViewController` constructor for caller-supplied HTTPS URIs only without browser-data access or external URL-handler behavior; D59/B65 adds only iOS single-precision vector addition through `vDSP_vadd`, with no portable math contract; D60/B66 adds only an iOS CommonCrypto SHA-256 wrapper for row 018, with no portable `framework-crypto` contract; D61/B67 adds only the iOS ModelIO extension-level import-support query, without file parsing or a portable contract; D62/B68 adds only an iOS MPS preferred-device availability query, without GPU work, operation support, or a portable contract; D63/B69 adds one borrowed P-256 public-key contract and Security algorithm-suitability query, without signature verification or key persistence; D64/B70 adds a portable finite SpriteKit node-position contract and iOS detached-`SKNode` position facade only, without SceneKit or rendering; D76/B72 adds a platform-exclusive CallKit call-count/state snapshot; D77/B73 adds portable finite map geometry with a MapKit backend; D85/B75 adds caller-app FileProvider registered-domain presence and B82 adds its count; B76 adds only ProximityReader Tap to Pay device-model support. D90 finds MarketplaceKit’s installation-source and region queries are async Swift-only and do not establish distribution readiness; D91 initially found MatterSupport’s API-support query was Swift-only; B233 now implements only request-API support and does not establish setup readiness; D92 finds SecureElementCredential’s Swift-only eligibility query requires a privileged entitlement and approved NFC & SE Platform access; D93 finds CarKey has no generic support query and its own Swift status values require an entitled session; D94 found ProximityReader’s Swift-only predicate reports only a device-model value; B76 now exposes that Boolean through a compiler-derived synchronous `swiftcall` shim, without claiming payment readiness. D95 finds LockedCameraCapture is an extension lifecycle and content handoff, not a general support query; D96 identifies public bundle metadata readers but no implemented helper, D97 initially found no proven WidgetKit Rust route; B254 now exposes a synchronous `reloadAllTimelines()` request only, while provider/timeline/rendering APIs remain unsupported. D98 matches ActivityKit call signatures but not owned-result linkage, and D99 reconfirms no stable App Intents metadata input. The canonical
[`capability-status.json`](docs/capabilities/capability-status.json) has 114 rows across 16
families: 36 portable rows are implemented and 22 are partial; iOS support is partial for 98 rows
(`B`) and unsupported in this workstream for 16 (`X`). No row is complete at the whole-capability
level. The manifest contains no `R`, `M`, `A`, or `C` rows. Portable counts include value contracts
without an iOS backend. All 16 remaining `X` rows now name a specific scope or toolchain gap; no generic missing-facade/backend reason remains. `X` means no supported implementation in the current
workstream, not that Apple's API is unavailable. B274 adds an activated-session count of previously
selected AccessorySetupKit accessories to row 044; B291 adds only the `PhotogrammetrySession.isSupported`
hardware predicate and B298 adds its two device-specific input limits to row 095; B280 adds AlarmKit
authorization state as new row 114. The focused
plans retain each slice's deployment, entitlement, host-configuration, ABI, and runtime limits. Row 008 also includes B305’s main-thread, iOS 8+ Increase Contrast setting snapshot and B309’s main-thread, iOS 8+ Color Filters/Grayscale preference-state snapshot; neither changes row coverage or claims rendered-color behavior. B313 adds a main-thread, iOS 13+ Auto-Play Video Previews setting snapshot without reporting app playback behavior. B317 adds an iOS 10+ typed hearing-aid paired-ear status snapshot with unknown values preserved and no identity, connection, or audio-route claim. B325 adds an iOS 7+ borrowed-view frame-to-screen coordinate conversion for `accessibilityFrame`; B330 adds an iOS 7+ path-to-screen conversion returning a new retained path. Neither mutates the view or claims presentation.

Row 100 ExtensionKit/Foundation remains `X`: D86/B221 found only a Swift-only, asynchronous inventory for
a host-defined extension point, with no process-readiness signal; iOS host/browser UI also needs
extension lifecycle integration. Row 102 ContactProvider remains `X`: D87/B224 found no public
Objective-C/C route or generated binding for its Swift-only `isEnabled` state, and enablement may
prompt. Row 001 has B81 partial support for a point-in-time connected-scene activation-state count; this does not deliver lifecycle events or prove visibility. Row 099 FileProvider has B75 partial support for caller-app registered-domain presence and B82 adds its count; locked host/device/Simulator checks, strict Clippy, rustdoc, and the link/import audit passed. Neither query proves enablement, sync, service availability, or file access. B84 adds an iOS-only `UIPasteboard.hasStrings` preflight without reading text; B85 adds an opaque plain-bookmark create/resolve value with implicit security scope omitted and no access grant; B86 preserves the raw signed local-notification authorization status without changing portable D3 semantics; B87 adds a prompt-free iOS 14+ full/reduced location-accuracy authorization snapshot and preserves unknown signed values without changing portable D4 or row 037 coverage; B88 adds a prompt-free raw alert/sound/badge settings snapshot without changing portable D3 or row 031 coverage; B90 adds an iOS-only regular-file size snapshot without changing the portable `framework-files` contract or row 014 coverage; B91 adds one prompt-free raw snapshot of additional local-notification presentation settings without changing D3 or row 031 coverage; B92 adds UIKit Adjustable and B95 adds UIKit NotEnabled accessibility traits; B93 adds an iOS-only one-path FileKind lookup under AppPath; B94 adds raw CarPlay and optional Siri-announcement settings but does not expose the preview-privacy choice or prove connection/presentation readiness. B96 adds an iOS-only no-follow POSIX modification-time snapshot under one AppPath; filesystem precision may be coarser than nanoseconds, and the value is not a content version or change token; B97 adds an unmanaged Background Assets queue count only; B98 adds UIKit KeyboardKey metadata only, with no keyboard or key-event callback; B99 adds a no-follow `(st_dev, st_ino)` snapshot for near-time comparison only, not a persistent ID or retained handle. B100 adds UIKit UpdatesFrequently metadata for a frequently changed label/value, with no poll or cadence promise; B102 adds UIKit PlaysSound metadata, with host-supplied activation sound and no audio action; B104 adds UIKit CausesPageTurn metadata, which requires host `accessibilityScroll:` behavior and a page-content update; B106 adds UIKit StartsMediaSession metadata only when activation starts a media session. B101 is a no-go for `st_birthtime` entry creation time because it may contain `ctime` with no per-entry support signal; B359 adds a distinct mutable stored `ATTR_CMN_CRTIME` value only after a same-descriptor volume support check. B365 adds `ATTR_VOL_SPACEUSED` as volume-wide used bytes, not app usage or quota. Both `fgetattrlist` slices require an applicable approved File Timestamp reason in host `PrivacyInfo.xcprivacy`; see `PLAN_IOS_FILE_CREATION_TIME.md` and `PLAN_IOS_VOLUME_USED_CAPACITY.md`. B105 adds `st_nlink` for regular files only, not alias paths or exclusive ownership. B107 adds no-follow `st_ctime` status-change seconds/nanoseconds, rejects final symlinks, and is not a content version or reliable change token. B109 adds no-follow `st_blocks` allocation units of 512 bytes for regular files only, with no exact physical-use or exclusive-allocation claim. B110 adds an iOS 26.0+ count of entries parsed from caller-supplied Background Assets manifest JSON, not local status or host setup. B113 copies only identifier, download size, and version from those entries into deterministically sorted owned descriptors; B116 also copies optional `userInfo` JSON bytes into Rust-owned data. Neither reports local or server state. B112 adds no-follow `st_atime` access-time seconds/nanoseconds, rejects final symlinks, and is not a guaranteed access log or reliable change token. B114 adds a synchronous read of `UIView.isAccessibilityElement`, reporting property state only, not visibility, focus, or assistive-app reachability. B117 adds nullable-property presence checks for accessibility label, hint, and value; they return no text and do not prove caller assignment. B115 adds raw no-follow `st_flags` with named `UF_*`/`SF_*` masks, preserves unknown bits, and does not predict effective access. B118 adds raw no-follow `st_uid`/`st_gid` values without account-identity or access claims; B121 is a no-go for `st_blksize`, an I/O size hint with no matching buffer or policy hook in the facade; B122 is a no-go for `st_rdev` on unsupported special entries; B125 is a no-go for superuser-only `st_gen`. B128 adds `IosFiles::directory_entry_count` without a Rust-owned name list or per-entry kind lookup; it is O(n), libc may allocate, and concurrent changes can affect the result. B131 adds an early-exit `IosFiles::directory_is_empty` query, not a stable snapshot or deletion guard. B134 adds fixed-width regular-file/directory/`Other` counts without copying names; a name that vanishes between enumeration and no-follow lookup yields an error, not partial counts. B137 adds a checked `f_bavail * f_bsize` capacity snapshot for the retained semantic root; it is volume-wide, not an app quota or write guarantee, and requires a host `PrivacyInfo.xcprivacy` Disk Space reason matching use. B146 adds checked `f_blocks * f_bsize` total mounted-volume capacity from the retained semantic root, not physical-device capacity or an app quota; it shares the same host privacy-manifest requirement. B140 declines Foundation `NSURLDirectoryEntryCountKey` as a cheap-count API because it is optional and filesystem-dependent; the descriptor API can be expensive, only 32-bit, and not supported by every volume format. B143 declines `f_bfree` because it includes reserved blocks unavailable to the sandbox; B137 already reports the non-superuser `f_bavail` capacity. B149 declines `f_ffree` because free filesystem nodes are volume-wide, not an app/container budget or a reliable prediction that a sandbox create will succeed. B116 adds optional Rust-owned Background Assets manifest `userInfo` JSON bytes from caller-supplied input; B119 found no additional safe read-only field in the installed SDK/binding, and `language` is beta for iOS 27. B124 adds an iOS 16.4+ asynchronous count of returned queue entries whose `BADownload.isEssential` is true; it is not an installed-asset or all-device count. B127 adds an iOS 16.1+ count of queue entries whose typed `priority` differs from `BADownloaderPriorityDefault`, without reading state or identifiers. B120 adds known-trait membership checks without raw mask exposure; B123 reads/writes whether child accessibility elements are hidden, not the receiver's own element flag, visual state, or interaction state; absent guarded selectors return `AccessibilityApiUnavailable` without raising the iOS 4.0 package floor. B126 adds a selector-guarded group-children property for logical parent groups; B129 adds a selector-guarded modal-child property for content that is actually modal. B132 adds guarded BCP 47 accessibility-language metadata without input normalization or dynamic callback behavior; B135 adds guarded interaction-response metadata but no interaction handler or assistive behavior claim. B138 adds typed navigation-style access; unknown native enum values remain unexposed, and the SDK documents the property for Switch Control only, not VoiceOver. B141 adds selector-guarded iOS 13.0 textual-context metadata using named constants only; it does not invoke the iOS 17 callback or claim assistive output. B144 adds guarded `Unspecified`/`List`/`Landmark` container types; B379 adds `SemanticGroup` with a caller iOS 13+ guard; B382 adds typed main-thread image-size adjustment get/set for `UIImageView`/`UIButton` with an iOS 11+ caller guard; `DataTable` remains excluded because it needs the data-table protocol. B147 declines `accessibilityDirectTouchOptions` because the VoiceOver passthrough/audio behavior needs a dedicated interaction contract. Neither claims assistive traversal or app/view presentation modality. B108 adds UIKit AllowsDirectInteraction metadata for caller-marked content that supports direct touch; the adapter routes no input or DirectTouchOptions. B111 adds UIKit SummaryElement metadata for a caller-supplied summary, with no presentation-timing claim. B97 changes row 036 to partial; the remaining listed B slices extend already-partial rows and do not change row coverage.
B153 adds selector-guarded iOS 5 `accessibilityActivationPoint` get/set over screen-coordinate `CGPoint` values; callers must recompute after layout/screen-position changes, and no assistive activation outcome is promised. B156 adds iOS 11 attributed-label set/presence operations; UIKit copies the value and couples it to `accessibilityLabel`, with no dynamic callback or speech claim. B159 adds selector-guarded iOS 11 attributed-hint set/presence; UIKit copies values and couples it to `accessibilityHint`, with no callback or speech claim. B162 adds selector-guarded iOS 11 attributed-value set/presence; UIKit copies values and couples it to `accessibilityValue`, with no callback or speech claim. B169 adds selector-guarded iOS 13 attributed user-input-label set/presence and preserves primary-first ordering without recognition guarantees. B171 adds selector-guarded iOS 7 `accessibilityPath` set/presence using screen-coordinate `UIBezierPath`; callers own geometry updates, with no highlight or activation claim. B174 adds selector-guarded screen-coordinate `accessibilityFrame` get/set over `CGRect`; it promises no coordinate conversion, layout, visibility, assistive output, or `UIAccessibilityElement` container-space behavior for `UIView`. B177 adds selector-guarded iOS 5 `accessibilityIdentifier` get/set for UI Automation only; it does not validate uniqueness or claim assistive output. The container-space frame property is excluded because it belongs to `UIAccessibilityElement`, not this `UIView` adapter. B173 confirms HomeKit status requires a manager instance whose first use can prompt; B176 records that DeviceActivity Boolean getters lack documented semantics, prompt/thread guarantees, and query-only entitlement semantics. B150 adds guarded iOS 13 `accessibilityUserInputLabels` set/presence operations; empty arrays are passed explicitly, UIKit may supply defaults, and the adapter makes no recognition or speech-match claim. B130/B133 copy optional remaining-byte allowances only from the typed `BAAppExtensionInfo` callback value (iOS 16.1/16.4); B136 classifies the typed `BAContentRequest` value and preserves unknown native values. B152 reports only the volume mount’s `MNT_RDONLY` bit, not effective app/path writability; B155 returns `f_iosize` as a sizing hint only, not an alignment or performance guarantee. B164 queries `_PC_NAME_MAX` for direct child components of one retained app-directory root, not total/nested path length or later-operation success. B170 reads the current effective UID’s no-follow `ATTR_CMN_USERACCESS` mask for one entry; it is a point-in-time OS permission report, not a later-operation guarantee or full access decision. B172 exposes cached Foundation `RENAME_EXCL`/`RENAME_SWAP` volume support as `Option<bool>`; the actual rename remains authoritative. B175 exposes cached Foundation case-sensitive and case-preserved volume name-support values as `Option<bool>`; it does not change path comparison semantics. B165 declines `_PC_PATH_MAX` because descriptor-relative `openat` walks components rather than passing one full path; B168 declines `F_GETPROTECTIONCLASS` because no public numeric class mapping exists and Foundation string values would require path/URL re-resolution. B158 declines `f_fsid`, `f_owner`, `f_type`, `f_fssubtype`, and `f_fstypename` as app-facing volume identity because they do not define a persistent/container identity or stable iOS filesystem mapping. B161 declines `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_QUOTA`, and `MNT_DONTBROWSE` as effective path/app authorization signals; B152 exposes only the read-only mount bit. B180 adds the iOS 10 `TabBar` trait for ordered tab lists; callers separately set `isAccessibilityElement` to false. B183 declines iOS 17 `ToggleButton` because the unconditional generated static has no per-trait availability result for supported iOS 12–16 targets. B186 adds selector-guarded iOS 18 `AccessibilityExpandedStatus` get/set, maps unknown native getter values to `None`, and leaves the callback block and actual content expansion to the caller. B189 declines `accessibilityElements` because it replaces and retains an untyped child array without a safe mixed-element ownership/cleanup or container-state contract. B192 adds a selector-guarded point-in-time per-view focus Boolean; B195 adds sorted Rust-owned focused assistive-technology identifiers with native nil preserved; B198 declines the global focused-element function because it may return an arbitrary retained object; B201 copies the iOS 13 user-input-label array while preserving nil/empty and native order; B204 copies the full accessibilityLanguage value while preserving nil/empty and without normalization; B207 copies the full textual-context value without normalization or callback invocation. B178 declines `NSURLVolumeMaximumFileSizeKey` until its `NSNumber` integer/Boolean type conflict is resolved; B181 declines a volume hard-link flag without a hard-link operation; B184 declines a symbolic-link support flag without a symlink operation; B187 declines an advisory-locking flag without a lock/open-handle operation; B190 declines the sparse-file support flag without a sparse allocation/hole operation. These are partial-row extensions and do not change row coverage. B196 adds an iOS-only regular-file copy-on-write clone using fclonefileat without replacing an existing destination; B199 adds a cached NSURLVolumeSupportsFileCloningKey hint, not a per-operation guarantee. B202 declines NSURLVolumeSupportsAccessPermissionsKey without a descriptor-safe setter, and B205 declines NSURLIsExcludedFromBackupKey because the documented setter re-resolves a URL rather than using the validated descriptor. B208 adds a no-follow regular-file ATTR_CMNEXT_EXT_FLAGS snapshot from an open descriptor, preserving raw bits and making no specific-clone-peer or durable-allocation claim. B211 adds an opaque ATTR_CMNEXT_CLONEID value for pure-clone ID comparison only, not a hash, version, or sharing proof. B214 adds an ATTR_CMNEXT_CLONE_REFCNT u32 point-in-time full-clone count, not partial peers, IDs, or paths; the host needs an applicable approved File Timestamp reason in PrivacyInfo.xcprivacy. B223 adds ATTR_CMNEXT_PRIVATESIZE as bytes not trapped in clone/snapshot data, not allocated size or future-space assurance; the host needs an approved File Timestamp reason. B225 adds selector-guarded accessibilityAttributedValue as an owned retained immutable value, without callback or speech/output claims. B228 adds the selector-guarded attributed user-input-label array getter; B234 returns an owned copy of the screen-coordinate accessibility path, preserving nil; B243 returns the raw accessibility-trait mask including unknown bits; B246 returns the selector-guarded iOS 17 direct-touch option mask without enabling touch behavior; B249 snapshots whether VoiceOver is running and B252 snapshots whether Switch Control is running with main-thread proof, no observer or UI action, and a caller iOS 8+ availability requirement because the imported C symbol has no runtime guard; B255 returns `None` when Guided Access is off to avoid AssistiveTouch’s documented false-value ambiguity, and otherwise returns the guarded iOS 10+ status. B226 adds ATTR_CMNEXT_LINKID for current-mounted-volume comparison only, not cross-mount persistence, content identity, inode, clone group, or authorization; the host needs an approved File Timestamp reason. B231 declines ATTR_CMNEXT_RELPATH because it is a mount-relative path with hard-link ambiguity and no facade operation; B232 declines ATTR_CMNEXT_ATTRIBUTION_TAG because no public bundle-ID mapping, app-data contract, or setter exists. B235 adds `ATTR_DIR_ALLOCSIZE` for the directory object only; B242 adds `ATTR_FILE_TOTALSIZE` as logical bytes across a regular file’s forks, not an allocation measure or a fork-content API; B244 adds `ATTR_FILE_DATAALLOCSIZE` for data-fork allocation only, excluding resource forks; B248 adds `ATTR_FILE_RSRCALLOCSIZE` for resource-fork allocation only, without exposing fork contents; B251 adds `ATTR_FILE_RSRCLENGTH` for resource-fork logical length only, without exposing fork contents; B247 declines `ATTR_FILE_FORKCOUNT` because no fork-enumeration or fork-operation contract consumes the count; B237/B238/B240/B241 decline logical directory size, directory I/O block size, directory link count, and mount status because no supported operation consumes those values; B247 declines `ATTR_FILE_FORKCOUNT` because no fork-enumeration or fork-operation contract consumes the count; B253 declines `ATTR_FILE_ALLOCSIZE` because B109/B244/B248 already provide the supported aggregate/data/resource allocation values without a further operation needing a duplicate all-fork scalar. B220 declines ATTR_CMNEXT_RECURSIVE_GENCOUNT because only APFS directories marked maintain-dir-stats yield meaningful values and no app-callable mark operation exists. B210/B213/B216 add selector-guarded plain accessibility label/hint/value getters with nil/empty preservation and Rust-owned text; B219/B222 return owned retained immutable attributed label/hint values. These getters do not invoke dynamic callback blocks or claim speech/output.
See [D86](PLAN_CAPABILITIES_EXTENSIONKIT.md), [D87](PLAN_CAPABILITIES_CONTACTPROVIDER.md), and
[D85/B75](PLAN_CAPABILITIES_FILEPROVIDER.md).

Row 103 ManagedApp/Distribution remains `X`: D89/B227 found Swift-only configuration and catalog APIs, an ambiguous nil configuration result, and entitlement-gated distribution operations; no generic managed-device status facade exists ([D89 audit](PLAN_CAPABILITIES_MANAGEDAPP.md)). Row 104 MarketplaceKit remains `X`: D90/B230 found no supported Rust async call path for its Swift-only installation-source and region values; these do not establish entitlement, Apple approval, or installation/vending eligibility ([D90 audit](PLAN_CAPABILITIES_MARKETPLACEKIT.md)). Row 105 MatterSupport is partial (`B`) for B233’s iOS 17+ `MatterAddDeviceRequest.isSupported` query only; it does not establish generic Matter, Thread, accessory, entitlement, ecosystem, or setup readiness ([D91 audit and B233 implementation](PLAN_CAPABILITIES_MATTERSUPPORT.md)). Row 106 SecureElementCredential remains `X`: D92/B270 found `CredentialSession.isEligible` is an iOS 18.1+ Swift actor async-throws getter with no supported synchronous Rust bridge, and SecureElementCredential calls without `com.apple.developer.secure-element-credential` raise `fatalError`; this is not an unentitled entitlement check or generic NFC/Secure Element/credential-presence query. Useful sessions still require Apple approval and user consent ([D92/B270 audit](PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md)). Row 107 CarKey remains `X`: D93 found no general Rust-callable CarKey support query; its own status values are session-bound and require the restricted MFi automaker entitlement ([D93 audit](PLAN_CAPABILITIES_CARKEY.md)). Row 108 ProximityReader is partial (`B`): B76 calls only `PaymentCardReader.isSupported` through a compiler-derived C `swiftcall` thunk; it reports the iPhone XS-or-newer device-model predicate, not iOS version, entitlement, region, merchant, PSP, reader, or payment readiness ([D94 audit](PLAN_CAPABILITIES_PROXIMITYREADER.md), [B76 plan](PLAN_IOS_PROXIMITYREADER.md)). Row 109 LockedCameraCapture remains `X`: D95 found no public support query, and B269 confirmed the closest `sessionContentURLs` value is an owned Swift `[URL]` array without a safe adapter in the current Rust ABI; it reports host content, not support or readiness. Capture still requires host configuration, camera authorization, and user-added controls ([D95/B269 audit](PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md)). Row 110 App Intents remains `X`: D99 reconfirmed no documented stable metadata input or processor API. Row 111 WidgetKit is partial (`B`) for B254's iOS 14+ `WidgetCenter.shared.reloadAllTimelines()` request; it neither invokes providers nor guarantees render timing or success. Configured providers, timelines, SwiftUI rendering, and widgets remain unsupported. Row 112 ActivityKit is partial (`B`) for B209's iOS 16.1+ ActivityAuthorizationInfo.areActivitiesEnabled snapshot; it reports only whether the current app can start a Live Activity now, requires host NSSupportsLiveActivities configuration to offer Live Activities, and does not implement create/update/end, event updates, or WidgetKit presentation ([D98/B209 plan](PLAN_CAPABILITIES_ACTIVITYKIT.md)). Row 113 is partial (`B`) for B77's caller-selected `NSExtension.NSExtensionPointIdentifier` runtime read only; build-host plist generation, `.appext`, schema validation, and App Intents remain unsupported. See [D96](PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md), [D97](PLAN_CAPABILITIES_WIDGETKIT.md), [D98](PLAN_CAPABILITIES_ACTIVITYKIT.md), and [D99](PLAN_CAPABILITIES_APP_INTENTS.md). Row 101 BrowserEngineKit remains `X` even though `objc2-browser-engine-kit` has generated process
and grant bindings: `isValid()` applies only to an already-owned live grant, not entitlement or
engine support. Useful operation needs browser process/XPC lifecycle, Swift-only extension
protocols, Apple-issued entitlements, and regional approval ([D88/B218 audit](PLAN_CAPABILITIES_BROWSERENGINEKIT.md)).

The largest remaining `X` family is extension/entitlement capabilities (8), followed by
commerce/services (2) and Sensors/connectivity (2), then cloud/accounts/communication and
Maps/AR/spatial (1). Graphics/GPU has no remaining `X` row. ML/vision/language and several smaller families have fewer
remaining rows. The 16 remaining `X` rows retain specific scope or toolchain gaps; rows 021, 074, 098, and 113 are partial rather than unsupported: B78 implements only an entitlement-scoped credential-state query for one prior Sign in with Apple user ID. Passkeys, general sign-in readiness, and the original non-entitled query scope remain unsupported ([D66 audit and B78 follow-up](PLAN_CAPABILITIES_PASSKEYS.md)).
Other gaps include PushToTalk's absence of a standalone support/authorization query and its entitlement, APNs, microphone, channel, and audio lifecycle requirements ([D83/B179 feasibility audit](PLAN_CAPABILITIES_PUSHTOTALK.md)); CarPlay's lack of a general availability query and required entitled host scene/session lifecycle; B182 confirms `supportsVideoPlayback` is only a connected-session property ([D84/B182 feasibility audit](PLAN_CAPABILITIES_CARPLAY.md)); HomeKit row 072 is partial only for B339's host-supplied per-accessory `HMAccessory.supportsIdentify` bit; B173/B322 still exclude prompt-free manager authorization and reachability snapshots, while B373 declines `isBlocked` without a documented thread-safety contract,
PushKit's token/service/delegate
and the full CallKit provider/call lifecycle ([D70 feasibility audit](PLAN_CAPABILITIES_PUSHKIT.md)), BackgroundAssets' host
extension/configuration boundary; row 036 has B97 only for unmanaged queue count ([D72/B97 plan](PLAN_CAPABILITIES_BACKGROUND_ASSETS.md)),
SensorKit requires an Apple-approved research entitlement; B203 reconfirms the only per-sensor authorization query uses a deprecated `SRSensorReader`, while beta `SRReader<Sensor>` is absent from the installed SDK/binding ([D68/B203 feasibility audit](PLAN_CAPABILITIES_SENSORKIT.md)),
AccessorySetupKit's absent non-prompting query, and DockKit's remaining async-sequence surface after B245 added a narrow system-setting snapshot ([D78/B197 audit and B245 implementation](PLAN_CAPABILITIES_DOCKKIT.md)), and TipKit's Swift-only per-tip eligibility
status that does not prove presentation ([D73 feasibility audit](PLAN_CAPABILITIES_TIPKIT.md)), plus two
compiler/build-host rows with specific scope/toolchain reasons: App Intents remains `X`, while WidgetKit is partial for B254's timeline-reload request; ActivityKit is partial for B209's current-app start-eligibility snapshot. B236 exposes only the default Foundation Models readiness Boolean through a compiler-derived Swift ABI bridge; sessions, inference, generation, and other model APIs remain unsupported
([D69 audit and B236 implementation](PLAN_CAPABILITIES_FOUNDATION_MODELS.md)). DeviceActivity's public
Objective-C authorization getter lacks documented query, prompt, thread, and entitlement semantics
([D71 feasibility audit](PLAN_CAPABILITIES_DEVICE_ACTIVITY.md)). Extension-bundle metadata row 113 is now a platform-exclusive `B` partial for the B77 runtime reader; build-host plist generation, `.appext`, point-specific schema validation, and App Intents remain out of scope. B239 exposes only AdAttributionKit's iOS 18+ app-impression support Boolean through compiler-derived `swiftcall`, while
AdServices token generation is an online operation rather than a status check
([D74 feasibility audit](PLAN_CAPABILITIES_AD_ATTRIBUTION.md)). D75 found no honest non-entitled Thread status facade. B206 now implements only the entitlement-scoped iOS 16.4+ `THClient.isPreferredNetworkAvailableWithCompletion:` query; it requires `com.apple.developer.networking.manage-thread-network-credentials` and Apple distribution approval, and does not report local radio or border-router capability. The Swift-only `MatterAddDeviceRequest.isSupported` query covers Matter setup, not Thread ([D75/B206 plan](PLAN_CAPABILITIES_THREAD.md)). D78/B197 is the historical DockKit feasibility audit. B245 now exposes only the iOS 17+
`DockAccessoryManager.shared.isSystemTrackingEnabled` setting through compiler-derived `swiftcall`;
Simulator returns `NativeApiUnavailable`, and no accessory presence, active tracking, camera state,
or event lifecycle is claimed. Row 097 is partial (`B`) for that setting only
([D78/B197 audit and B245 implementation](PLAN_CAPABILITIES_DOCKKIT.md)). D79/B250 found the native WeatherKit API is Swift-only and the REST availability endpoint is callable only as coordinate/data-set availability, not general readiness. A host-selected REST package still needs a trusted signed developer token, typed data contract, and attribution; row 091 remains `X` ([D79/B250 audit](PLAN_CAPABILITIES_WEATHERKIT.md)). D80/B194 found RealityKit scene/entity operations remain Swift-facing while `ARView` exposes only a view shell; row 095 remains `X` ([D80/B194 feasibility audit](PLAN_CAPABILITIES_REALITYKIT.md)). D82 identified a narrow Personal VPN status path; B79 implements a read-only caller-app profile-status snapshot after preference load, and B200 adds only the loaded configuration `isEnabled` and `isOnDemandEnabled` flags. Row 098 remains partial; provider lifecycle, preference mutation, tunnel control, and global VPN state remain unsupported. A false `isEnabled` can reflect another enabled configuration, and `isOnDemandEnabled` does not prove rules or automatic connection behavior ([D82 audit and B79/B200 plan](PLAN_CAPABILITIES_NETWORK_EXTENSION.md)). These gaps
differ: some await a bounded public facade; others depend on user
authorization, entitlement, Xcode metadata support, or future ABI evidence.

D13's portable `framework-data` byte and UTF-8 views are paired with B19's explicit raw-byte CFData
copy bridge in `PLAN_IOS_DATA.md`; native string conversion remains outside scope. D15 is implemented
in `framework-connectivity` as an informational one-shot network-path snapshot; B18 adds its
separate public Network.framework C backend in `PLAN_IOS_CONNECTIVITY.md`. Row 028 is
now counted as partial `B` support. G12 target gates are wired and passed locally as recorded in
`PLAN_VALIDATION_IOS_CONNECTIVITY.md`; no CI workflow run or live path behavior is claimed. This
snapshot does not preflight requests or claim endpoint reachability.

B20 adds `ios-url` as a strict Foundation adapter for the D8 `Uri` value. It uses only
`NSURL::URLWithString_encodingInvalidCharacters` with `false`, declares an iOS 17.0 API floor, and
retains the exact source `Uri`; Foundation acceptance and parser parity remain unverified. The
device and Simulator link/import probes and target compile/lint gates pass locally; see
`PLAN_IOS_URL.md` and `PLAN_VALIDATION_IOS_URL.md`.

D16 adds exact positive-area intersection for finite `framework-ui::Frame` values. B21 maps that
contract through private CoreGraphics C declarations in `ios-ui::geometry`, with an iOS 2.0 API
floor and exact `CoreGraphics` plus `libSystem.B.dylib` probe imports. Device/Simulator compile,
strict Clippy, C/Rust LP64 layout, and symbol-link checks pass locally; runtime CoreGraphics parity
remains unverified. See `PLAN_CAPABILITIES_GEOMETRY.md` and `PLAN_IOS_GEOMETRY.md`.

D17 adds `framework-media::MediaTime`, a no-std finite rational value with positive timescale and
exact `i128` comparison. B22 converts it to `objc2_core_media::CMTime` by value via the pinned
0.3.2 binding, with an iOS 4.0 API floor. Host/device/Simulator compile and strict Clippy plus
CoreMedia import and layout probes pass locally; media runtime, playback, capture, and AVFoundation
behavior remain outside scope. See `PLAN_CAPABILITIES_MEDIA.md` and `PLAN_IOS_MEDIA.md`.

D18 adds `framework-ui::FontMetrics` and a `TextMetricsBackend` contract for finite system-font
ascent, descent, and leading values. B23 implements one iOS query with public CoreText C APIs; the
SDK availability floor is iOS 3.2, while probe deployment targets are not product minimums. Device
and Simulator builds, strict Clippy, C signature/layout, and Release import checks pass locally.
Text shaping, width, line breaks, Dynamic Type, UIKit parity, and live CoreText output remain
outside scope. See `PLAN_CAPABILITIES_TEXT_METRICS.md`, `PLAN_IOS_TEXT_METRICS.md`, and
`PLAN_VALIDATION_IOS_TEXT_METRICS.md`.

D19/B24 implements outbound TLS-over-TCP only: one owned bounded send/read operation at a time, with a 1 MiB per-operation cap; listeners, UDP, Bonjour, HTTP, and WebKit remain outside scope. D20/B25 implements app-refresh scheduling only; scheduler timing is nondeterministic, host task identifiers and background modes remain app-owned, and no task launch/expiry runtime is claimed. D21/B26 implements metadata-only ImageIO source count and image-zero encoded dimensions; D22/B27 implements Photos read/write authorization only, preserving limited access separately from full access; D23/B28 implements one UIKit background-execution lease with cooperative expiry and explicit end; D24/B29 implements Contacts authorization status and request only, preserving Limited separately from full access; D25/B30 implements EventKit Calendar event-authorization status and explicit full-access request, preserving WriteOnly separately from FullAccess. See `PLAN_CAPABILITIES_CONNECTION.md`, `PLAN_IOS_CONNECTION.md`, `PLAN_CAPABILITIES_BACKGROUND_TASKS.md`, `PLAN_IOS_BACKGROUND_TASKS.md`, `PLAN_CAPABILITIES_IMAGE_METADATA.md`, `PLAN_IOS_IMAGE_METADATA.md`, `PLAN_CAPABILITIES_PHOTOS.md`, `PLAN_IOS_PHOTOS.md`, `PLAN_CAPABILITIES_BACKGROUND_EXECUTION.md`, `PLAN_IOS_BACKGROUND_EXECUTION.md`, `PLAN_CAPABILITIES_CONTACTS.md`, `PLAN_IOS_CONTACTS.md`, `PLAN_CAPABILITIES_CALENDAR.md`, and `PLAN_IOS_CALENDAR.md`.

D27/B32 adds an allocation-free authorization snapshot using `CBManager.authorization` on iOS 13.1+. D29/B34 adds an explicitly requested unfiltered foreground central scan, copied UUID/RSSI values, and a fixed 32-event FIFO. A scan may prompt, is asynchronous, and has no background guarantee; connection, peripheral/advertising operations, and radio controls remain out of scope. Discovery identity has an iOS 8.0 floor, while authorization remains iOS 13.1; see `PLAN_CAPABILITIES_BLUETOOTH.md`, `PLAN_IOS_BLUETOOTH.md`, and `PLAN_IOS_BLUETOOTH_DISCOVERY.md`.

D28/B33 adds the borrowed HTTPS URL and platform-exclusive navigation contract in `framework-web`, with a typed `WKWebView` adapter. It has an iOS 8.0 declaration floor and no JavaScript bridge, arbitrary file/HTML/data load, subresource firewall, browser parity, or page-load guarantee; see `PLAN_CAPABILITIES_WEB.md` and `PLAN_IOS_WEB.md`.

D30/B35 exposes only whether `NSFileManager.ubiquityIdentityToken` is present for iCloud Drive Documents. It never returns or retains the token; nil causes vary, and presence does not prove container access, synchronization, or CloudKit account status. See `PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md` and `PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md`.

D47/B52 adds only a one-shot CloudKit account-status snapshot from the app's default container. It preserves unknown fixed-width status values and an optional framework-owned error; it does not read data, observe account changes, expose account identity, or prove database/container access. The signed app requires CloudKit container and service entitlements. See `PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md` and `PLAN_IOS_CLOUDKIT_ACCOUNT_STATUS.md`.

D48/B53 adds only `SACrashDetectionManager.isAvailable`, a point-in-time Crash Detection device-support bit guarded at iOS 16.0. No portable contract is useful for this hardware/service predicate. The local SDK header states an entitlement is required for the class, but Apple does not specify whether this getter alone needs it; the signed app's event entitlement and authorization are not claimed. The slice does not receive crash events or provide emergency response. See `PLAN_CAPABILITIES_SAFETYKIT.md` and `PLAN_IOS_SAFETYKIT.md`.

D49/B54 adds `framework-maps::WorldTrackingSupport` and an iOS 11.0+ `ARWorldTrackingConfiguration.isSupported` query. The value reports configuration support only; it does not start a session, access the camera, request camera permission, or guarantee tracking. See `PLAN_CAPABILITIES_ARKIT.md` and `PLAN_IOS_ARKIT.md`.

D50/B55 adds `framework-game`'s point-in-time local-player authentication contract and reads only `GKLocalPlayer.localPlayer.isAuthenticated`. It does not initialize authentication, show UI, expose player identity/data, or observe changes. A configured app requires the signed Game Center entitlement. See `PLAN_CAPABILITIES_GAMEKIT_STATUS.md` and `PLAN_IOS_GAMEKIT_STATUS.md`.

D51/B56 adds only `MLModel.availableComputeDevices` non-emptiness from iOS 17.0. It does not load a model, run inference, expose device objects, or guarantee model compatibility. No portable contract is useful for this iOS Core ML hardware snapshot. See `PLAN_CAPABILITIES_COREML.md` and `PLAN_IOS_COREML.md`.

D52/B57 adds `framework-vision::TextRecognitionRevisionSupport` and checks only whether the runtime lists a caller-supplied revision for `VNRecognizeTextRequest`. The iOS 13.0+ query creates no request and reads no image; it does not establish recognition success or model readiness. See `PLAN_CAPABILITIES_VISION.md` and `PLAN_IOS_VISION.md`.

D53/B58 adds only a read of the app's saved Speech authorization status. It does not request permission, create a recognizer, accept audio, or start recognition; authorization does not establish service availability or recognition success. See `PLAN_CAPABILITIES_SPEECH_STATUS.md` and `PLAN_IOS_SPEECH_STATUS.md`.

D54/B59 checks only whether Apple’s built-in English contextual-model assets are on-device; it does not load a model, accept text, compute vectors, request assets, or guarantee a later model operation. See `PLAN_CAPABILITIES_NATURALLANGUAGE_STATUS.md` and `PLAN_IOS_NATURALLANGUAGE_STATUS.md`.

D55/B60 adds only the deprecated StoreKit 1 `SKPaymentQueue::canMakePayments` snapshot. It does not
create a queue, inspect a product/account, display UI, start a purchase, or process a transaction.
No portable commerce contract or StoreKit 2 product/transaction operation is included in D55; D58/B63
adds a separate status-only StoreKit 2 query. See
`PLAN_CAPABILITIES_STOREKIT_STATUS.md` and `PLAN_IOS_STOREKIT_STATUS.md`.

D56/B61 adds `framework-roomplan::RoomPlanDeviceSupport`, a portable allocation-free status value, and
`ios-roomplan::device_support()` for iOS 16.0+. The backend calls only `RoomCaptureSession.isSupported`
through a compiler-verified C `swiftcall` thunk; non-iOS targets return `None`. It does not create a
session, access sensor frames, request permission, or guarantee a scan. See
`PLAN_CAPABILITIES_ROOMPLAN.md` and `PLAN_IOS_ROOMPLAN.md`.

D57/B62 adds only an iOS 4.0+ default-video-device presence query through
`AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`. It does not query authorization,
create a capture input/session, access media, or present UI. See
`PLAN_CAPABILITIES_CAMERA_DEVICE_STATUS.md` and `PLAN_IOS_CAMERA_DEVICE_STATUS.md`.

D58/B63 adds only the iOS 15.0+ `AppStore.canMakePayments` status through a compiler-verified
weak-import `swiftcall` thunk. It does not retrieve products, access account/entitlement data, show
purchase UI, or process transactions. See `PLAN_CAPABILITIES_STOREKIT2_STATUS.md` and
`PLAN_IOS_STOREKIT2_STATUS.md`.

D59/B65 adds only single-precision vector addition through the iOS 4.0+ `vDSP_vadd` C API, with
equal-length borrowed slices and unit strides. It adds no portable math contract or other
Accelerate operations. See `PLAN_CAPABILITIES_ACCELERATE.md` and `PLAN_IOS_ACCELERATE.md`.

D60/B66 adds only an iOS 2.0+ one-shot SHA-256 call through Apple's `CC_SHA256` API, with an
explicit `CC_LONG` input bound and a fixed 32-byte result. It adds no portable `framework-crypto`
contract, key operation, Rust replacement, parity, or performance claim. See
`PLAN_CAPABILITIES_CRYPTO.md` and `PLAN_IOS_CRYPTO.md`.

D61/B67 adds only the iOS ModelIO extension-level `MDLAsset.canImportFileExtension` query. It does
not add asset loading, file access, parsing, rendering, GPU support, or a portable ML contract. See
`PLAN_CAPABILITIES_MODELIO_STATUS.md` and `PLAN_IOS_MODELIO_STATUS.md`.

D62/B68 adds only whether `MPSGetPreferredDevice` returns a device with default options. It does
not submit GPU work or establish support for an MPS operation, model, or workload. See
`docs/ios/mps-status.md`.

D63/B69 adds a borrowed uncompressed P-256 public-key value and an iOS query for whether Security
supports ECDSA/SHA-256 message verification with that key. The portable constructor checks only
the point marker; Security validates the point during import. This does not verify a signature,
generate or use private keys, persist keys, or access the Secure Enclave. See
`PLAN_CAPABILITIES_KEY_SUPPORT.md` and `PLAN_IOS_KEY_SUPPORT.md`.

D64/B70 adds a finite portable `SpriteNodePosition` contract and a detached iOS `SKNode` create,
read, and set facade for parent-local `position` only. Its API floor is iOS 7.0; the link probes use
12.0 device and 14.0 Simulator minimums for the installed SDK and repository baseline. SceneKit,
scene/view creation, rendering, node hierarchy, animation, and physics remain out of scope. See
`PLAN_CAPABILITIES_SPRITEKIT.md` and `PLAN_IOS_SPRITEKIT.md`.

D65/B71 reads the point-in-time MediaPlayer library authorization state on iOS 9.3+ only. It does
not request access, read media items, contact Apple Music services, or provide catalog/playback
support. The usage-description key is required for host access requests or item reads; this
status-only slice does neither. See `PLAN_CAPABILITIES_MEDIA_LIBRARY_STATUS.md` and
`PLAN_IOS_MEDIA_LIBRARY_STATUS.md`.

D76/B72 adds a platform-exclusive CallKit snapshot of active-call count and aggregate outgoing,
connected, on-hold, and ended flags. `CXCallObserver.calls` is read synchronously and may block; no
call object, UUID, caller data, callback, call control, provider, PushKit, or audio API is exposed.
See `PLAN_CAPABILITIES_CALLKIT.md` and `docs/ios/call-observer.md`.

D77/B73 adds portable finite map coordinates, projected map points, and non-negative distances,
with iOS conversion and distance via MapKit. It does not add map UI, user location, permission,
network, search, directions, or full MapKit parity. See `PLAN_CAPABILITIES_MAPKIT.md` and
`PLAN_VALIDATION_IOS_MAPKIT.md`.

D81/B74 reads only the caller-owned `NSUserActivity.isClassKitDeepLink` Boolean on iOS 11.3+; it
does not access the ClassKit store or assignment data. Schoolwork host adoption and the separate
ClassKit environment entitlement remain outside this query. See `PLAN_CAPABILITIES_CLASSKIT.md`.

D66 found no row-021 query that meets the original non-prompting, non-entitled boundary. B78 now implements one prior-user credential-state query under a conservative Sign in with Apple entitlement requirement, and row 021 is `B`/partial for that slice only; see `PLAN_CAPABILITIES_PASSKEYS.md` and `PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md`.

D67/B80 implements a raw signed iOS 15+ `AuthorizationCenter.shared.authorizationStatus` snapshot through a compiler-matched C `swiftcall` bridge. The unsafe API requires the main dispatch queue and preserves `approvedWithDataAccess` plus unknown future values without assigning enum meanings. It does not claim entitlement presence, distribution approval, activity-data access, or control use; query-only entitlement semantics remain undocumented. Row 074 is `B`/partial; see `PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md`.

D69 is a feasibility audit for row 067 Foundation Models. The smallest status candidate is a
Swift-only `SystemLanguageModel` property with no public C/Objective-C entry point or generated Rust
binding; a supported wrapper or Swift ABI boundary is not integrated. Row 067 is partial (`B`) for B236’s iOS 26.0+ `SystemLanguageModel.default.isAvailable` Boolean only; session creation, inference, generation, prompts, context size, supported languages, use-case support, and unavailable reasons remain unsupported. The API is beta and no model call or runtime parity was tested; see
`PLAN_CAPABILITIES_FOUNDATION_MODELS.md`.

D71 is a feasibility audit for row 075 DeviceActivity/ManagedSettings. A public iOS 17 Objective-C
`DeviceActivityAuthorization.isAuthorized` getter is callable in principle, but its Boolean meaning,
prompt behavior, thread guarantees, and query-only entitlement requirements are undocumented. Row
075 remains `X`; B376/B383 confirm that the Swift-only schedule/event queries expose host-configured monitoring data rather than general capability and lack a scoped Rust collection/value ABI contract; see `PLAN_CAPABILITIES_DEVICE_ACTIVITY.md`.

D73/B191 is a feasibility audit for row 092 TipKit. `Tip.status` is a Swift-only per-tip eligibility value,
not global TipKit readiness or proof of presentation; no public C/Objective-C entry point or generated
Rust binding is available. Row 092 remains `X`; see `PLAN_CAPABILITIES_TIPKIT.md`.

D74/B185 is a feasibility audit for row 089 AdAttributionKit/AdServices. B239 now implements only the iOS 18.0 `AppImpression.isSupported` scalar through a compiler-derived weak `swiftcall` thunk; this is a partial (`B`) request-specific API-support snapshot, not general AdAttributionKit readiness. AdServices token generation remains network-dependent and is not a status query; see `PLAN_CAPABILITIES_AD_ATTRIBUTION.md`.


D31/B36 adds camera and microphone authorization-status queries only through AVFoundation. It does
not request authorization, enumerate/select devices, create capture/audio sessions, or access
samples. The iOS API floor is 7.0; the host's camera/microphone usage descriptions apply before an
access request or capture, not this status-only query. See
`PLAN_CAPABILITIES_MEDIA_AUTHORIZATION.md` and `PLAN_IOS_MEDIA_AUTHORIZATION.md`.

D32/B37 adds a caller-owned NFC reader-support snapshot through
`NFCReaderSession.readingAvailable`, with an iOS 11.0 API floor. It creates no session or tag
operation; the guide conservatively records `NFCReaderUsageDescription` because Apple does not
document a property-only exception. See `PLAN_CAPABILITIES_NFC.md` and `PLAN_IOS_NFC.md`.

D34/B39 reports only Nearby Interaction's `supportsPreciseDistanceMeasurement` device capability
through `NISession.deviceCapabilities` on iOS 16.0+. It does not report permission, peer
compatibility, session readiness, or operation success; no session or ranging operation is run.
See `PLAN_CAPABILITIES_NEARBY_INTERACTION.md` and `PLAN_IOS_NEARBY_INTERACTION.md`.

D35/B40 reads only the calling app's App Tracking Transparency status. It never requests
authorization, reads an advertising identifier, or performs tracking; the iOS 14.0 floor and
`NSUserTrackingUsageDescription` host requirement are recorded in
`PLAN_IOS_TRACKING_AUTHORIZATION.md`.

D36/B41 reports only whether the public `MTLCreateSystemDefaultDevice` factory returns a device
object on iOS 8.0+. The retained object is dropped at once; no command queue, GPU work, MetalKit,
feature support, or performance claim is in scope. See `PLAN_CAPABILITIES_METAL.md` and
`PLAN_IOS_METAL.md`.

D37/B42 reports only DeviceCheck and App Attest API support bits; it does not create tokens or
keys, attest/assert, or establish trust. D38/B43 reports only whether `WCSession.isSupported()` is
true; it does not inspect pairing or communicate. D39/B44 reports only the empty/nonempty state of
`EAAccessoryManager.connectedAccessories` at query time; no hardware identity or communication
guarantee is exposed. D40/B45 reads only the legacy ReplayKit `RPScreenRecorder.isAvailable` value
on iOS 9.0+ and starts no capture or recording; Apple marks that property deprecated and recommends
ScreenCaptureKit, which remains outside this slice. D41/B46 reports only whether the built-in
SoundAnalysis version 1 classifier request is recognized on iOS 15.0+; it creates no analyzer,
supplies no audio, requests no microphone access, and does not cover ShazamKit.
D42/B47 reports whether any other app is playing audio through `AVAudioSession.isOtherAudioPlaying`
on iOS 6.0+. The value includes ambient-category audio and does not identify an app or item, report
this app's playback, or expose Now Playing metadata or media control. See
`PLAN_CAPABILITIES_OTHER_AUDIO.md` and `PLAN_IOS_OTHER_AUDIO.md`.

D43 (`PLAN_CAPABILITIES_PLAYBACK.md`) adds one portable HDR-eligibility snapshot in
`framework-audio`; B48 reads only `AVPlayer.eligibleForHDRPlayback` on the iOS main thread and does
not inspect an asset or start playback. D44 (`PLAN_IOS_COMMUNICATION_STATUS.md`) records
platform-exclusive MessageUI and SharedWithYou status bits without a portable contract; B49 reads
only `canSendMail`, `canSendText`, and `isSystemCollaborationSupportAvailable` through dedicated
iOS crates. D45/B50 adds `VideoCodecType` and `HardwareDecodeSupport` (see `docs/capabilities/video-codec.md`); `ios-media` reads only `VTIsHardwareDecodeSupported` from iOS 11.0 and does not encode, create decoder sessions, process frames, or reserve resources. See `PLAN_IOS_VIDEOTOOLBOX.md`. D46/B51 reads only `PKPaymentAuthorizationController.canMakePayments` from iOS 10.0; it does not inspect cards or process payments. D47/B52 reads only `CKContainer.accountStatus` from iOS 8.0; it does not access CloudKit data, and the signed app requires CloudKit container/service entitlements.

These are bounded slices, not full platform parity. Remaining rows need their own scoped contracts
and evidence; compile/lint checks do not establish live native behavior, and the installed Xcode
26.6 / iOS SDK 26.5 evidence remains below the planned Xcode 27.x baseline. Current scope excludes,
among other areas, remote push, background processing/continued execution beyond one app-refresh
path, Network.framework listeners/UDP/Bonjour, full browser parity, full
window/scene/navigation/layout/accessibility systems, and general graphics or sensor frameworks
beyond the specific rows recorded as `B`.

## Capability status manifest

Create a machine-readable and human-readable support manifest with, for every capability:
- crate/module;
- portability class;
- iOS implementation class R/M/B/A/C/X;
- minimum iOS version;
- required frameworks;
- required Info.plist keys;
- entitlements;
- device-only restrictions;
- Swift ABI requirement;
- current parity/performance status;
- native escape handle availability.

The manifest drives docs and linkage tests; do not duplicate inconsistent support tables manually.

## Platform extensions

If an Apple feature has no cross-platform equivalent:
- put it under `ios::...`;
- do not pollute the portable type with Apple classes;
- common API may still expose shared state/result where meaningful.

## Async/cancellation

Every async capability must specify:
- when the operation starts;
- cancellation guarantee versus cancellation request only;
- completion exactly-once behavior;
- whether dropping Rust future cancels, detaches, or only drops interest;
- callback thread/queue;
- reentrancy.

Do not hide background/system lifecycle limits.

## Error semantics

Portable error category plus iOS detail.
Do not convert every NSError/OSStatus to text only.
Preserve status/domain/code in platform extension.

## Documentation

Every capability must have:
- short rustdoc example;
- support/availability;
- permissions/entitlements;
- lifecycle/threading;
- error behavior;
- cancellation;
- cost/copy/allocation notes when important;
- Apple backend or Rust replacement rationale;
- native escape instructions.

## Tests

Portable:
- semantic state/error/cancellation tests;
- no platform types leak;
- no_std build.

iOS:
- capability integration tests;
- permissions mocked/injected where feasible;
- simulator/device split clearly marked.

Linkage:
- minimal example per capability family verifies no unrelated frameworks.

## Non-goals

- no universal lowest-common-denominator model;
- no fake portability for Apple-exclusive capabilities;
- no claim of unsupported entitlement access;
- no SwiftUI/Flutter-style renderer;
- no silently degraded semantics.

## Handoff

Produce final support matrix showing every researched Apple framework/capability mapped to:
- implemented portable capability,
- implemented iOS-only capability,
- Apple-backed backend,
- benchmark-selected Rust backend,
- or explicitly documented V1 unsupported reason.
