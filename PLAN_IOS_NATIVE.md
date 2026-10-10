# PLAN_IOS_NATIVE.md — Workstream B: iOS Native Runtime and Backends

## Status

## Completed tooling baseline for outstanding B work

R1/R2 build/validation tooling is installed and source-isolated. B implementations use the pinned PATH `ios-rust-build` and `ios-rust-validate` contracts described in `docs/SHARED_TOOLING.md`, without implementing compiler/SDK discovery or validation orchestration themselves. Add a declarative native `build-spec.json` and minimal Cargo bridge **only if** the particular capability actually compiles C sources; Rust-only/objc2 crates must not gain pointless build scripts. Preserve existing focused shell/ABI/import tests when the configured validator does not yet cover their capability, then migrate only with demonstrable positive and negative parity. Tool source fixes belong to separate maintenance; product API contracts, packaging, and real-device tests remain B work.


The minimal Rust-owned UIKit app builds for the arm64 device and simulator targets, and the
unsigned Xcode archive gate passes on Xcode 26.6 / SDK 26.5, below the Xcode 27.x plan baseline.
An added x86_64 simulator bundle launched on iOS 18.0 and showed the adaptive-color label/button
in light and dark appearance. A launch-argument smoke also sent `UIControlEvents::TouchUpInside`
through `UIControl::sendActionsForControlEvents`; the Rust callback changed the label to
`Rust callback: 1 tap(s)`. This is supplemental x86_64 evidence of programmatic target/action
dispatch only: it does not verify arm64 runtime or a user touch. Capability backends retain their
own subplan status and limits.

On 2026-10-09, `sh examples/ios-minimal/build.sh simulator` and `sh examples/ios-minimal/build.sh device` both passed. `file` confirms arm64 Mach-O executables; `xcrun vtool -show-build` reports `IOSSIMULATOR` and `IOS` respectively, each with minos 17.0 and SDK 26.5. These are Release build/package checks only; this run did not launch either app or create an archive.

## Objective

Implement the native iOS runtime substrate and capability-scoped Apple C/CoreFoundation/Objective-C backends entirely from Rust/C/Objective-C ABI tooling, with no Swift application/source layer.

## Dependencies

Requires integrated `PLAN_FOUNDATION.md`.

## Execution decomposition

The native backend matrix is too broad for one executor. B1 is `PLAN_IOS_APP_DATA.md`: it implements the iOS files and preferences backends only after their D1 portable contracts are integrated. B2 is `PLAN_IOS_SECURE_STORAGE.md`: it implements the Keychain backend after D2. B3 is `PLAN_IOS_NETWORK.md`: it implements foreground HTTP after D1. B4 is `PLAN_IOS_NOTIFICATIONS.md`: it implements the local-notification backend after D3. B5 is `PLAN_IOS_LOCATION.md`: it implements one-shot current location after D4. B6 is `PLAN_IOS_CLIPBOARD.md`: it implements the UIKit general-pasteboard backend after D5. B7 is `PLAN_IOS_SHARE.md`: it implements the UIKit outgoing-share backend after D6, including the retained callback session required by F7. B8 is [PLAN_IOS_ACCESSIBILITY.md](PLAN_IOS_ACCESSIBILITY.md): it adds bounded accessibility metadata setters for caller-owned UIKit views. B9 is [PLAN_IOS_PRESENTATION.md](PLAN_IOS_PRESENTATION.md): it adds a bounded one-action UIKit acknowledgement alert for a caller-owned presenter. B10 is [PLAN_IOS_RESOURCES.md](PLAN_IOS_RESOURCES.md): it reads ordinary packaged files from the main application bundle after D7. B11 is [PLAN_IOS_BROWSER.md](PLAN_IOS_BROWSER.md): it adds a bounded external HTTPS URL-handler request using `UIApplication.open(_:options:completionHandler:)`. B12 is [PLAN_IOS_NOTIFICATION_RESPONSES.md](PLAN_IOS_NOTIFICATION_RESPONSES.md): it adds an opt-in bridge from native response data to D9 values. B13 is [PLAN_IOS_BACKGROUND_TRANSFER.md](PLAN_IOS_BACKGROUND_TRANSFER.md): it implements Foundation URLSession background downloads after D10 and B14 are integrated. B14 is [PLAN_IOS_FILE_ADOPTION.md](PLAN_IOS_FILE_ADOPTION.md): it adds that narrow `ios-files` operation. B1 owns the D1 sandbox implementation; B14 and B17 are additive named extensions in `ios-files`, with root reconciling shared crate exports and indexes. B15 is [PLAN_IOS_MOTION.md](PLAN_IOS_MOTION.md): it implements the one-shot raw accelerometer backend after D11. B16 is [PLAN_IOS_AUTHENTICATION.md](PLAN_IOS_AUTHENTICATION.md): it implements one-shot local authentication after D12 with public LocalAuthentication APIs. B17 is [PLAN_IOS_FILE_COORDINATION.md](PLAN_IOS_FILE_COORDINATION.md): it adds synchronous caller-owned file URL read/write coordination; it does not change the sandbox `Files` contract. B18 is [PLAN_IOS_CONNECTIVITY.md](PLAN_IOS_CONNECTIVITY.md): it implements a one-shot informational network path snapshot after D15; it does not preflight or gate D1/B3 requests. B24 is [PLAN_IOS_CONNECTION.md](PLAN_IOS_CONNECTION.md): it implements outbound TLS-over-TCP streams for D19 through Network.framework C APIs; listeners and UDP are excluded. B25 is [PLAN_IOS_BACKGROUND_TASKS.md](PLAN_IOS_BACKGROUND_TASKS.md): it implements one BGAppRefreshTask path for D20; BGProcessingTask and UIKit background execution remain excluded. B26 is [PLAN_IOS_IMAGE_METADATA.md](PLAN_IOS_IMAGE_METADATA.md): it implements source count and image-zero encoded dimensions through ImageIO only. B27 is [PLAN_IOS_PHOTOS.md](PLAN_IOS_PHOTOS.md): it implements read/write authorization status and request through PhotoKit only. B28 is [PLAN_IOS_BACKGROUND_EXECUTION.md](PLAN_IOS_BACKGROUND_EXECUTION.md): it implements one UIKit background-execution lease with cooperative expiry and explicit end; app extensions are unsupported. B29 is [PLAN_IOS_CONTACTS.md](PLAN_IOS_CONTACTS.md): it implements Contacts authorization status and request only through the public Contacts framework. B30 is [PLAN_IOS_CALENDAR.md](PLAN_IOS_CALENDAR.md): it implements EventKit Calendar event-authorization status and explicit full-access request for iOS 17.0+. B31 is [PLAN_IOS_HEALTH_AUTHORIZATION.md](PLAN_IOS_HEALTH_AUTHORIZATION.md): it implements availability and explicit type-scoped read/share authorization only, without grant inference or health-data access. B32 is [PLAN_IOS_BLUETOOTH.md](PLAN_IOS_BLUETOOTH.md): it reads `CBManager.authorization` without manager creation or a prompt. B33 is [PLAN_IOS_WEB.md](PLAN_IOS_WEB.md), a bounded typed `WKWebView` HTTPS/navigation adapter with no browser-parity claim. B34 is [PLAN_IOS_BLUETOOTH_DISCOVERY.md](PLAN_IOS_BLUETOOTH_DISCOVERY.md), an explicitly requested unfiltered foreground central scan with copied peer UUID/RSSI values and a fixed event queue; it does not connect or expose peripherals. B35 is [PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md](PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md), a presence-only `NSFileManager.ubiquityIdentityToken` snapshot that does not expose the token or claim CloudKit status. B36 is [PLAN_IOS_MEDIA_AUTHORIZATION.md](PLAN_IOS_MEDIA_AUTHORIZATION.md), a status-only AVFoundation camera/microphone authorization query; it does not request access or capture media. B37 is [PLAN_IOS_NFC.md](PLAN_IOS_NFC.md), the Core NFC `readingAvailable` support query; it does not create a session or operate on tags. B38 HomeKit feasibility audit found that first `HMHomeManager` use can prompt, so no non-prompting status-only backend is claimed. B39 is [PLAN_IOS_NEARBY_INTERACTION.md](PLAN_IOS_NEARBY_INTERACTION.md), a non-prompting iOS 16+ precise-distance capability query with no session, token, peer, or ranging operation. Unlisted capability backend families need separate named subplans and must not overlap these owned paths.
 B76 is [PLAN_IOS_PROXIMITYREADER.md](PLAN_IOS_PROXIMITYREADER.md): it calls only `PaymentCardReader.isSupported` through a compiler-derived C `swiftcall` thunk and returns the iPhone device-model predicate; it does not establish payment readiness. B206 adds the ios-thread-network backend for the iOS 16.4+ preferred-network availability query; the host needs the ThreadNetwork entitlement and distribution approval, and this API reports no radio or border-router capability ([D75/B206](PLAN_CAPABILITIES_THREAD.md)).

Read first:
- `docs/IOS_BUILD.md`
- `docs/OBJC_INTEROP.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- `docs/APP_STORE_COMPLIANCE.md`
- `docs/research/NATIVE_CAPABILITY_MATRIX*.md`
- `docs/research/FRAMEWORK_ELIMINATION_AUDIT.md`

## Write scope

- `platform/ios/ios-runtime/**`
- capability-specific `platform/ios/ios-*` crates
- `examples/ios-minimal/**`
- capability-specific iOS integration tests
- iOS backend docs under `docs/ios/**`

Do not edit shared core contracts except through central reconciliation.

## iOS runtime substrate

### `ios-runtime`
Proposed internal modules/symbols:

- `autorelease`
- `main_thread`
- `objc_class`
- `block`
- `delegate`
- `availability`
- `ns_error`
- `cf`
- `darwin`
- `dispatch`
- `native_handle`

Use `objc2`/`block2` as preferred current dependencies, with narrow adapters and no public leakage into portable APIs.

Proposed wrappers:
- `IosErrorDetail`
- `ObjcOwned<T>` only internally if `objc2::Retained<T>` cannot remain local enough
- `MainThread`/marker adapter that composes with `objc2::MainThreadMarker`
- callback context primitive using compact IDs only where direct object ownership cannot suffice
- panic-catching FFI boundary helper

Do not reimplement `objc_msgSend`, ARC, Blocks, or generated bindings absent a measured/ABI gap.

## Minimal app vertical slice

Implement an iOS example proving:
- Rust-owned app entry/linkage;
- `UIApplication`;
- Rust-defined app delegate;
- `UIWindow`;
- `UIViewController`;
- `UILabel`;
- `UIButton`;
- target/action or delegate callback into Rust;
- main-thread typing;
- object teardown;
- Release simulator/device build;
- no Swift source.

This slice is the acceptance gate before broad UIKit wrappers.

## Native capability backend rules

For each capability:
1. select the smallest current public Apple boundary;
2. prefer C/CoreFoundation/Darwin when it is simpler and semantically sufficient;
3. otherwise use Objective-C via generated `objc2` binding;
4. keep application state/protocol parsing/algorithms in Rust;
5. convert at the boundary only;
6. expose native escape access under iOS-specific extension modules;
7. document permissions/Info.plist/entitlements/lifecycle.

## Capability backend families

Implement the following native/system backends, capability-scoped so linking one does not pull the others:

### Application/UI
- UIKit application/window/view/controller primitives;
- native controls/events;
- clipboard/pasteboard;
- share/presentation helpers;
- accessibility/native traits where public API permits;
- no virtual DOM/custom renderer.

### Files/preferences
- POSIX/Darwin ordinary file I/O where appropriate;
- Foundation only for Apple-specific directory/resource/coordination semantics;
- `NSUserDefaults`.

### Secure storage/security/authentication
- Security/Keychain;
- SecKey/Secure Enclave public APIs where requested;
- LocalAuthentication;
- AuthenticationServices: B78 implements only prior-user Sign in with Apple credential-state status under a conservative entitlement requirement; passkeys and authorization flows remain unsupported;
- DeviceCheck/App Attest;
- AppTrackingTransparency/AdServices native paths where applicable.

### Network
- current Network framework C/native route where useful;
- URLSession foreground;
- URLSession background file downloads implemented as B13 after D10 and B14; they remain separate from B3;
- SystemConfiguration only for still-supported semantics;
- B33 covers one platform-exclusive `WKWebView` view/navigation slice. It does not claim browser parity or include a JavaScript bridge; B64 separately provides one host-presented `SFSafariViewController` constructor for HTTPS. B11 covers only the external UIKit URL-handler request.
- D19/B24 now scopes only outbound TLS-over-TCP byte streams through public Network.framework C APIs, separately from D1/B3 URLSession HTTP and D15/B18 informational NWPathMonitor status. It does not include listeners, UDP, Bonjour, or endpoint preflight; those remain unplanned without a bounded consumer need.

### Notifications/background
- UserNotifications;
- PushKit;
- BackgroundTasks;
- UIKit background task APIs;
- BackgroundAssets where public/native;
- no fake replacement for OS scheduling.

### Sensors/radios/hardware
- CoreLocation;
- CoreMotion;
- CoreBluetooth;
- CoreNFC;
- NearbyInteraction;
- SensorKit where available;
- ExternalAccessory;
- AccessorySetupKit;
- ThreadNetwork.

### Camera/audio/media
- AVFoundation capture;
- AVFAudio/CoreAudio/AudioToolbox I/O;
- CoreMedia/CoreVideo;
- VideoToolbox;
- MediaPlayer;
- ReplayKit;
- ShazamKit/SoundAnalysis native public paths.

### Graphics/GPU
- CoreGraphics;
- QuartzCore;
- CoreText;
- CoreImage;
- ImageIO;
- Metal/MetalKit;
- MPS/MPSGraph;
- Accelerate;
- ModelIO;
- SpriteKit/SceneKit only as platform extensions, not portable core assumptions.

### ML/vision/language
- Core ML generic model API;
- Vision;
- NaturalLanguage Apple-model features;
- Speech.

### Personal/system stores
- Photos;
- Contacts;
- EventKit;
- HealthKit;
- HomeKit;
- SafetyKit;
- ScreenTime native pieces;
- DeviceActivity/ManagedSettings only where public native exposure exists; Swift-only residuals belong to C.

### Cloud/accounts/system services
- CloudKit;
- iCloud native surfaces;
- MessageUI;
- SharedWithYou;
- WatchConnectivity;
- GameKit;
- ClassKit;
- PassKit/Apple Pay;
- CarPlay;
- CallKit;
- PushToTalk.

### Maps/AR/spatial
- MapKit;
- ARKit;
- Objective-C-exposed RealityKit pieces such as `ARView`;
- RoomPlan/DockKit native pieces where public bindings permit.

### Extension/entitlement shells
- NetworkExtension/VPN: B79 scopes a read-only caller-app Personal VPN profile-status query; provider extensions, tunnel control, and system-wide VPN state remain unsupported;
- B77 reads only `NSExtension.NSExtensionPointIdentifier` from one caller-selected `.appex`; no build-host plist generation, `.appext`, or App Intents support is claimed;
- ExtensionKit native pieces;
- FileProvider;
- BrowserEngineKit for eligible apps;
- ContactProvider native pieces;
- Managed app/distribution native surfaces;
- MatterSupport Objective-C extension handler;
- LockedCameraCapture native AVFoundation/camera engine pieces.

## Ownership and callbacks

Required invariants:
- use `Retained<T>`/borrowed/weak semantics correctly;
- do not put Objective-C objects inside `Arc` merely for sharing;
- delegates/targets are narrow Rust-defined Objective-C classes;
- callbacks may reenter;
- stale callbacks are impossible or detected;
- no retain cycles;
- no panic across Objective-C/C callback boundary;
- Block copy/dispose behavior matches Apple semantics;
- main-thread-only APIs require typed proof.

## Error mapping

Preserve native NSError/OSStatus/domain/code in cold platform detail while mapping stable portable error categories.

Do not collapse all native errors to strings.

## Build/package support

Using Xcode is allowed/required for:
- bundle construction;
- signing;
- provisioning;
- entitlements;
- assets;
- archive/export.

No Swift source compilation may be required by framework source.

Record exact macOS/Xcode toolchain manifest at execution start.

## Tests

For each backend:
- object lifecycle stress;
- callback exactly once where API promises it;
- reentrancy;
- cancellation;
- wrong-thread behavior where testable;
- availability gates;
- error mapping;
- minimal-link audit.

Use simulator for broad coverage; physical device where capability/hardware requires it.

## Non-goals

- no Swift-only residual implementation;
- no reimplementation of Apple system-owned services;
- no custom renderer;
- no replacing hardware-accelerated APIs on theory;
- no private APIs.

## Handoff

For every capability report:
- Apple framework/public API used;
- backend crate;
- linked framework set;
- permissions/entitlements;
- tests;
- remaining Swift-only symbols handed to Workstream C.
