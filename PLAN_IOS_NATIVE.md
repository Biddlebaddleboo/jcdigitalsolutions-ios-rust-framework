# PLAN_IOS_NATIVE.md — Workstream B: iOS Native Runtime and Backends

## Objective

Implement the native iOS runtime substrate and capability-scoped Apple C/CoreFoundation/Objective-C backends entirely from Rust/C/Objective-C ABI tooling, with no Swift application/source layer.

## Dependencies

Requires integrated `PLAN_FOUNDATION.md`.

## Execution decomposition

The native backend matrix is too broad for one executor. B1 is `PLAN_IOS_APP_DATA.md`: it implements the iOS files and preferences backends only after their D1 portable contracts are integrated. B2 is `PLAN_IOS_SECURE_STORAGE.md`: it implements the Keychain backend after D2. B3 is `PLAN_IOS_NETWORK.md`: it implements foreground HTTP after D1. B4 is `PLAN_IOS_NOTIFICATIONS.md`: it implements the local-notification backend after D3. B5 is `PLAN_IOS_LOCATION.md`: it implements one-shot current location after D4. B6 is `PLAN_IOS_CLIPBOARD.md`: it implements the UIKit general-pasteboard backend after D5. B7 is `PLAN_IOS_SHARE.md`: it implements the UIKit outgoing-share backend after D6. B8 is [PLAN_IOS_ACCESSIBILITY.md](PLAN_IOS_ACCESSIBILITY.md): it adds bounded accessibility metadata setters for caller-owned UIKit views. B9 is [PLAN_IOS_PRESENTATION.md](PLAN_IOS_PRESENTATION.md): it adds a bounded one-action UIKit acknowledgement alert for a caller-owned presenter. B10 is [PLAN_IOS_RESOURCES.md](PLAN_IOS_RESOURCES.md): it reads ordinary packaged files from the main application bundle after D7. Other capability backend families require separate named subplans and must not overlap these owned paths.

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
- AuthenticationServices passkeys/Sign in with Apple;
- DeviceCheck/App Attest;
- AppTrackingTransparency/AdServices native paths where applicable.

### Network
- current Network framework C/native route where useful;
- URLSession foreground;
- URLSession background retained for OS-managed background transfer;
- SystemConfiguration only for still-supported semantics;
- WebKit/SafariServices.

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
- NetworkExtension/VPN;
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
