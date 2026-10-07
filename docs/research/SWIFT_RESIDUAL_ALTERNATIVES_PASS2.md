# Swift Residual Alternatives — Pass 2

Research date: 2026-10-07

This pass covers additional specialized Swift-first frameworks and asks whether their unique capability actually justifies Swift ABI/tooling work.

The main pattern continues: specialized Apple frameworks are often genuine Swift residuals, but many are entitlement-limited or replaceable by broader native frameworks for ordinary application needs. They therefore should not drive the foundational ABI layer.

---

# CarKey

## Capability

Apple's CarKey framework provides remote-keyless access to vehicles already provisioned in Wallet.

Core public types include:

- `CarKeyRemoteControl`
- `CarKeyRemoteControlSession`
- `CarKeyRemoteControlSessionDelegate`
- Swift value types such as `VehicleReport`, actions and identifiers

Reference:
https://developer.apple.com/documentation/carkey

## Access restriction

Apple states that use requires:

```
com.apple.developer.carkey.session
```

and that requesting the entitlement requires being an **automaker enrolled in the MFi Program**.

This is not a general consumer-app car-key API.

## Native alternatives

For ordinary accessory/vehicle communication, apps may use:
- CoreBluetooth;
- ExternalAccessory where applicable;
- Network;
- vendor cloud APIs.

None reproduces Wallet CarKey remote-control integration, but they cover general vehicle connectivity without this restricted entitlement.

## Classification

The CarKey API is a genuine Swift residual for automaker apps.

Likely ABI needs:
- Swift classes;
- Swift delegate/protocol conformance;
- action/value structs;
- errors;
- callbacks/async depending on operation.

**S2 / specialized P3 for the general framework.**

Do not let CarKey shape Layer-1 Swift ABI design because almost no ordinary third-party application can use it.

---

# SecureElementCredential

## Capability

SecureElementCredential is Apple's public framework for managing applets/credentials in the device Secure Element and supporting wired/contactless transactions.

Apple describes support for:
- in-store payments;
- car keys;
- transit;
- corporate/student IDs;
- home/hotel keys;
- loyalty/rewards;
- event tickets.

Reference:
https://developer.apple.com/documentation/secureelementcredential

## Core API is strongly Swift-native

Apple exposes:

```swift
actor CredentialSession
```

as the primary entry point.

That immediately introduces:
- Swift actor isolation;
- Swift concurrency;
- nested Credential values;
- async operations;
- Swift errors/value types.

The framework also provides SwiftUI and UIKit extensions for user-facing wired/contactless operations.

## Entitlement and provisioning

Use requires the Secure Element Credential entitlement and an applet bundle registered through Apple Business Register.

Reference:
https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.secure-element-credential

This is another restricted/specialized integration.

## UIKit does not eliminate the core Swift session

The presence of UIKit transaction UI helpers does not make credential management Objective-C-native. The underlying `CredentialSession` remains a Swift actor.

Therefore:
- presentation may have native UIKit hooks;
- credential lifecycle and secure-element operations remain a Swift-ABI/concurrency problem.

## Classification

**S2, specialized P2/P3.**

It is more ABI-demanding than a normal Swift class because actor isolation and concurrency semantics matter.

Do not bypass actor semantics through private Secure Element interfaces.

---

# WidgetKit: app-side control is much easier than widget definition

## Executive finding

WidgetKit has a useful split:

1. **App-side widget management** through `WidgetCenter` is a relatively simple Swift API.
2. **Defining/rendering widgets** requires generic timeline/provider protocols and SwiftUI views.

The framework can support (1) substantially earlier than (2).

## WidgetCenter

Apple exposes:

```swift
class WidgetCenter
```

with simple operations such as:

```swift
reloadTimelines(ofKind:)
reloadAllTimelines()
getCurrentConfigurations(...)
```

Reference:
https://developer.apple.com/documentation/widgetkit/widgetcenter

This is likely a **small Layer-1 S1/S2 surface**:
- class singleton;
- Swift String;
- callback/result containing WidgetInfo values.

No Rust-defined Swift protocol type is needed merely to tell existing widgets to reload.

## Widget definition remains difficult

Widget timelines use:

```swift
Timeline<EntryType>
where EntryType : TimelineEntry
```

and providers conform to:
- `TimelineProvider`;
- `AppIntentTimelineProvider`;
- `IntentTimelineProvider`.

Reference:
https://developer.apple.com/documentation/widgetkit/timeline

Apple explicitly states widgets use SwiftUI views for display and WidgetKit renders those views in another process.

Reference:
https://developer.apple.com/documentation/widgetkit/keeping-a-widget-up-to-date

Therefore full Rust-defined widget support requires:
- Rust-backed Swift TimelineEntry/provider types;
- protocol/witness metadata;
- extension/build metadata;
- likely SwiftUI-compatible view generation or another supported rendering path;
- App Intents integration for modern configurable widgets.

## Recommendation

Split WidgetKit into two deliverables:

### WidgetKit Control API

Support early if useful:
- reload timelines;
- inspect current configurations;
- invalidate recommendations.

Classification: **S1/S2 Layer 1**.

### Widget Extension Authoring

Defer until:
- Layer 2 Rust-defined Swift types work;
- App Intents metadata is understood;
- a supported UI strategy is established.

Classification: **S2/S3 + Layer 4**, low priority.

This avoids making widget rendering block useful app-side widget control.

---

# MatterSupport

## Capability

MatterSupport coordinates adding Matter accessories to an application's ecosystem.

Apple exposes:

```swift
struct MatterAddDeviceRequest
class MatterAddDeviceExtensionRequestHandler
```

Reference:
https://developer.apple.com/documentation/mattersupport

The request includes topology, optional Matter setup payload, device criteria, and user-facing commissioning flow.

Apple's examples use `MTRSetupPayload`, which comes from the Matter framework.

Reference:
https://developer.apple.com/documentation/mattersupport/adding-matter-support-to-your-ecosystem

## Relationship to Matter/HomeKit

Do not confuse:
- the low-level/public Matter stack;
- HomeKit;
- MatterSupport's system-mediated device-add flow.

MatterSupport's unique value is system commissioning/ecosystem coordination. Low-level Matter device communication may have other C/Objective-C routes, but they do not necessarily reproduce this add-device UX/authorization flow.

## ABI shape

Likely requires:
- Swift request structs;
- arrays/topology values;
- Codable-like value semantics;
- extension request handler subclass/override surface;
- errors;
- UIKit/system presentation.

The extension-handler aspect may require Layer-2 Swift type/class interop and packaging.

## Classification

**S2/S3 specialized P2/P3.**

Research in depth only for apps implementing their own Matter ecosystem.

Ordinary HomeKit/Matter consumers should first use HomeKit/native APIs where those meet the requirement.

---

# DockKit

## Capability

DockKit controls compatible motorized tracking stands and integrates with camera apps.

Apple exposes:

```swift
final class DockAccessory
class DockAccessoryManager
```

with async methods and many Swift value types.

References:
https://developer.apple.com/documentation/dockkit
https://developer.apple.com/documentation/dockkit/dockaccessory

## API characteristics

Operations include:
- selecting/tracking subjects;
- orientation control;
- animations;
- region-of-interest;
- battery/accessory/tracking state.

State/event streams use concrete `AsyncSequence` types such as:
- StateChanges;
- MotionStates;
- BatteryStates;
- TrackingStates.

Reference:
https://developer.apple.com/documentation/dockkit/dockaccessory/motionstates-swift.struct

## Native alternative

AVFoundation/Vision can perform camera capture and subject detection, but they cannot command a DockKit-certified stand through the same system integration.

Therefore DockKit has a real unique capability.

## Classification

**S2 category A / P3 specialized.**

It mainly consumes Apple-defined Swift classes/value types and async sequences, so foundational Layer-1 ABI primitives should be reusable here.

Do not implement general AsyncSequence first; bridge concrete DockKit streams when needed.

---

# RoomPlan

## Capability

RoomPlan provides higher-level room/structure reconstruction on top of LiDAR/ARKit.

Apple exposes:
- `RoomCaptureSession` class;
- `RoomCaptureSessionDelegate : AnyObject` Swift protocol;
- `CapturedRoom` struct;
- `CapturedStructure` struct;
- `RoomBuilder` / `StructureBuilder`.

References:
https://developer.apple.com/documentation/roomplan
https://developer.apple.com/documentation/roomplan/roomcapturesessiondelegate

## ARKit is not an equivalent replacement

ARKit is native/Objective-C-accessible and can provide tracking/world geometry, but reproducing RoomPlan's semantic room reconstruction would require significant custom modeling.

Therefore:
- ordinary AR: use ARKit R2;
- Apple semantic room capture: RoomPlan is a genuine residual.

## Swift delegate requirement

`RoomCaptureSessionDelegate` is documented as:

```swift
protocol RoomCaptureSessionDelegate : AnyObject
```

rather than an `@objc` protocol.

That means a Rust-backed delegate likely requires Layer-2 Swift protocol conformance rather than objc2's Objective-C protocol machinery.

## Value model

`CapturedRoom` and `CapturedStructure` are Swift structs with nested Swift structs/enums/arrays.

Example:
- rooms;
- floors;
- walls;
- windows;
- objects;
- semantic sections.

References:
https://developer.apple.com/documentation/roomplan/capturedstructure
https://developer.apple.com/documentation/roomplan/capturedroom/surface

## Async post-processing

`StructureBuilder.capturedStructure(from:)` is async throwing.

Reference:
https://developer.apple.com/documentation/roomplan/structurebuilder/capturedstructure(from:)

Therefore RoomPlan requires:
- Layer-1 Swift values/async;
- Layer-2 protocol conformance for direct capture-session callbacks.

## Classification

**S2 Layer 1+2 / P3 specialized.**

RoomPlan should not shape initial ABI primitives, but is a useful later validation of Rust-defined Swift delegates and nested data/value access.

---

# Cross-pass conclusions

## Swift-only frameworks often divide into three practical levels

### Level A — simple control surface

Example:
- WidgetCenter

May need only a handful of Swift calls and standard values.

### Level B — Apple-defined Swift domain API

Examples:
- DockKit
- SecureElementCredential
- ProximityReader
- WorkoutKit

Requires Swift value/async support but not necessarily Rust-defined domain protocols.

### Level C — Rust must define system-facing Swift types/protocols

Examples:
- RoomPlan delegate
- ActivityKit attributes
- GroupActivity
- MatterSupport extension handler
- full WidgetKit providers

Requires Layer-2 type metadata/conformance work.

This distinction should determine implementation order.

## Entitlement rarity should lower priority

CarKey and SecureElementCredential are important platform capabilities, but their restricted access makes them poor foundational test cases.

## Native adjacent frameworks remain the first fallback

Before using a specialized Swift-only framework:

- CarKey -> CoreBluetooth/ExternalAccessory/vendor API for non-Wallet vehicle control
- SecureElementCredential -> Security/CoreNFC/PassKit when those capabilities suffice
- MatterSupport -> HomeKit/Matter native surfaces when system commissioning isn't required
- DockKit -> AVFoundation/Vision for tracking without motorized accessory control
- RoomPlan -> ARKit for general AR/geometry

Only use Swift ABI where the specialized framework's unique system capability is actually required.
