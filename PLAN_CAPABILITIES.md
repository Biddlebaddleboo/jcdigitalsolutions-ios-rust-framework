# PLAN_CAPABILITIES.md — Workstream D: Portable Capability Facades and V1 iOS Coverage

## Objective

Build the high-level developer-facing Rust capability API and assemble the complete V1 iOS backend from Workstreams B/C/E.

The portable API models semantic capabilities, not Apple class names.

## Dependencies

Requires Foundation contracts from A.
Consumes iOS native implementations from B, Swift residual implementations from C, and benchmark-selected replacements from E.

## Execution decomposition

Workstream D is too broad for one bounded executor. Start with `PLAN_CAPABILITIES_APP_DATA.md` (D1), which owns the application lifecycle, files, preferences, foreground HTTP facades, and the first complete support manifest. `PLAN_CAPABILITIES_SECURE_STORAGE.md` (D2) owns secure storage. `PLAN_CAPABILITIES_NOTIFICATIONS.md` (D3) owns the bounded local-notification contract. Further capability groups require separate named subplans and executors; they must not overlap D1-owned crates or manifest edits.

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
- App Intents: use result of C Stage 0/1; if unsupported, expose no fake runtime implementation.
- WidgetKit: support management/data logic available through proven interfaces; do not implement a SwiftUI clone merely to claim full rendering support.
- ActivityKit: support if Layer-2 ABI work is proven.
- extension bundle metadata helpers: only through supported Xcode/public mechanisms.

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
