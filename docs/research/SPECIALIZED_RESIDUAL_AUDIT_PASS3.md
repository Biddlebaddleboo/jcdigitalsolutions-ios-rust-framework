# Specialized Residual Audit — Pass 3

Research date: 2026-10-07

This pass moves further down the practical-frequency curve. The goal is to keep specialized frameworks in the durable census without allowing niche Swift-heavy APIs to distort the core architecture.

## Summary

| Framework | Result | Class | Priority |
|---|---|---|---|
| DockKit | Genuine Swift class API, but specialized accessory control; AVFoundation integration remains native | S2 | P3 |
| RoomPlan | Swift-first high-level scanning/model API; ARKit/AVFoundation/CoreML remain native alternatives for custom pipelines | S2 | P3 |
| RealityKit | Hybrid: ARView is explicitly @objc, but core Entity/component ecosystem is Swift-first | Hybrid R2/S2 | P2-P3 |
| TipKit | Hybrid: TipUIView is @objc/UIKit, but defining Tip values/rules uses Swift protocols/async streams | R2 + S2 | P3 |
| Core Transferable | Swift protocol/generic abstraction over transfer capabilities already available through older native APIs | D/P3 | Defer |
| JournalingSuggestions | SwiftUI picker + Swift value/protocol model, special entitlement | S2/S4 | P3 |
| ContactProvider | Swift protocol-based extension model | S2/S3-ish | P3 |
| ExtensionFoundation | Swift protocol-based generic app-extension entry model | S2/S3 | P3 |
| ManagedApp / ManagedAppDistribution | Swift-first enterprise/MDM frameworks with special deployment constraints | S2 | P3 |
| MarketplaceKit | Swift-first and region/entitlement/review restricted | S2 | P3 |
| TabletopKit | Swift-first, visionOS/SharePlay/RealityKit specialized | S2 | P4 for iOS core |

# DockKit

Apple exposes:

```swift
final class DockAccessory
```

for controlling compatible motorized camera stands.

The framework integrates with `AVCaptureSession`, which is already native Objective-C accessible.

References:
https://developer.apple.com/documentation/dockkit
https://developer.apple.com/documentation/dockkit/dockaccessory

## Strategy

Treat DockKit as an optional Swift-ABI package.

The actual camera pipeline should remain AVFoundation/objc2.

Rust-side subject detection/ML can remain pure Rust/Core ML/Vision; only accessory control crosses Swift ABI.

## Priority

**P3** due accessory specialization.

# RoomPlan

RoomPlan is a high-level room-scanning framework built around AR and structured room models.

The framework is valuable for room-scanning apps, but it is not a foundational iOS capability.

## Strategy

Before implementing RoomPlan ABI for a general app:

1. determine whether ARKit/RealityKit/Vision/AVFoundation native paths already satisfy the app;
2. only use RoomPlan where Apple's semantic room reconstruction is specifically needed.

Classification: **S2 specialized**.

Priority: **P3**.

# RealityKit

## Important hybrid finding

Apple declares:

```swift
@MainActor @objc @preconcurrency
class ARView
```

Reference:
https://developer.apple.com/documentation/realitykit/arview

Therefore the UIKit surface for presenting a RealityKit scene is Objective-C compatible.

However, core scene/model objects such as:

```swift
class Entity
```

are Swift-native and participate in RealityKit's component/entity ecosystem.

Reference:
https://developer.apple.com/documentation/realitykit/entity

## Strategy

Do not classify RealityKit as uniformly Swift-only.

Possible staged support:

1. **R2** ARView presentation/integration.
2. Continue using ARKit/Metal/SceneKit native APIs where they satisfy the application.
3. Add **S2** RealityKit entity/component ABI only for apps that specifically need RealityKit semantics.

## Priority

**P2-P3**, depending on AR/spatial application demand.

# TipKit

## Hybrid finding

Apple exposes:

```swift
@MainActor @objc @preconcurrency
final class TipUIView
```

for UIKit presentation.

Reference:
https://developer.apple.com/documentation/tipkit/tipuiview

But an application-defined tip normally conforms to the Swift `Tip` protocol and uses Swift rule/status/update APIs.

Therefore:
- display container: R2;
- tip definitions/rules/status streams: S2.

## Strategy

For a Rust-first framework, a simple app hint system can stay completely Rust/UIKit and avoid TipKit.

Implement TipKit only when the app specifically wants Apple's system-managed TipKit eligibility/state behavior.

Priority: **P3**.

# Core Transferable

Apple describes Core Transferable as a Swift-focused protocol abstraction.

The underlying capabilities already have public native alternatives:

- UIPasteboard;
- drag/drop;
- UIDocument/file APIs;
- share sheets;
- item providers;
- UTType/UniformTypeIdentifiers.

## Strategy

Do not implement `Transferable` solely to make Rust applications capable of moving data.

Use native transfer APIs first.

Only add Swift `Transferable` interoperability when another Apple API specifically requires a conforming type.

Classification: **D/P3**.

# Journaling Suggestions

Apple explicitly says to declare `JournalingSuggestionsPicker` using SwiftUI.

The picker is:

```swift
struct JournalingSuggestionsPicker<Label> where Label: View
```

with an async completion closure.

References:
https://developer.apple.com/documentation/journalingsuggestions
https://developer.apple.com/documentation/journalingsuggestions/journalingsuggestionspicker

The framework also uses Swift suggestion/asset values and requires a special entitlement.

## Strategy

This is exactly the kind of UI-heavy Swift capability that should stay near the bottom of the list.

Do not let it drive SwiftUI/result-builder support.

Priority: **P3**.

# ContactProvider

Apple's Contact Provider framework exposes a protocol-based extension model:

- `ContactProviderExtension`;
- `ContactProviderDomain`;
- `ContactItemEnumerating`;
- `ContactItemEnumerator`;
- Swift enum/item values.

Reference:
https://developer.apple.com/documentation/contactprovider

This is not equivalent to ordinary Contacts access. The normal Contacts framework remains native/objc2.

## Strategy

Only apps that need to inject an independently managed read-only contact database into the system need ContactProvider.

Supporting it likely requires:
- Rust-defined Swift protocol-conforming extension type;
- extension packaging;
- Swift value/enumerator types.

Classification: **S2/S3-ish**.

Priority: **P3**.

# ExtensionFoundation

Apple's modern extension entry model centers on:

```swift
protocol AppExtension
```

A concrete type must adopt the protocol and provide configuration/XPC behavior.

Reference:
https://developer.apple.com/documentation/extensionfoundation/appextension/

## Strategy

Do not make ExtensionFoundation a prerequisite for ordinary iOS extensions.

Many traditional extension points still use Objective-C-compatible principal classes/protocols or platform-specific frameworks.

Research ExtensionFoundation only for extension types that specifically require the modern AppExtension protocol model.

Classification: **S2/S3** because application-defined Swift-compatible types and extension discovery/configuration are involved.

Priority: **P3**.

# ManagedApp and ManagedAppDistribution

These frameworks target MDM/enterprise deployments.

Apple describes ManagedApp as an API for reading secure managed secrets/configuration provisioned by device management.

ManagedAppDistribution lets device-management solutions vend assigned managed apps and requires the **Managed App Installation UI** entitlement.

References:
https://developer.apple.com/documentation/managedapp
https://developer.apple.com/documentation/managedappdistribution

## Strategy

These are legitimate capability gaps for enterprise products, but they should not influence the general consumer framework architecture.

Implement only after Layer-1 Swift ABI support is mature and an MDM product needs them.

Priority: **P3**.

# MarketplaceKit

MarketplaceKit supports:
- alternative app marketplaces;
- web distribution;
- installation-source behavior.

Use is constrained by geography, Apple approval, entitlements, and alternative-distribution rules.

Reference:
https://developer.apple.com/documentation/marketplacekit

## Strategy

This framework is product-policy specific and should remain optional.

Do not make it part of the initial general-purpose framework surface.

Priority: **P3**.

# TabletopKit

TabletopKit is designed for spatial multiplayer tabletop games on visionOS and combines:
- Swift protocol/value types;
- SharePlay;
- RealityKit;
- spatial interaction.

Reference:
https://developer.apple.com/documentation/tabletopkit

For an iOS-first general framework this is extremely low priority.

Priority: **P4**.

# Frequency-based conclusion

At this point the research curve is flattening.

Most **high-frequency iPhone app capabilities** have already fallen into:
- pure Rust;
- C;
- Objective-C/objc2;
- a small Layer-1 Swift residual set.

As we move farther down the list, the remaining Swift-only frameworks increasingly have one or more of these properties:

- specialized hardware;
- restricted entitlements;
- enterprise/MDM use;
- extension-specific compiler integration;
- SwiftUI-heavy UI;
- visionOS/spatial focus;
- app-defined Swift protocol types.

That is favorable for the core architecture: a normal iOS app should be able to use the framework extensively without loading or paying for a broad Swift-interop subsystem.

# Updated practical priority bands

## Band 1 — common/native core

Already native:
- Foundation/UIKit;
- networking;
- storage/security;
- notifications;
- location;
- camera/media;
- Bluetooth/sensors;
- auth/passkeys;
- web;
- maps;
- contacts/photos;
- Core ML/Vision;
- Metal;
- CloudKit;
- background tasks;
- CallKit/PushKit;
- VPN/NetworkExtension;
- Apple Pay;
- most common system services.

## Band 2 — high-value Swift residuals

- StoreKit 2;
- Translation;
- Foundation Models;
- App Intents;
- AdAttributionKit;
- selective MusicKit capability;
- WidgetCenter management.

## Band 3 — specialized Swift residuals

- ActivityKit;
- GroupActivities;
- AlarmKit;
- WorkoutKit;
- ProximityReader;
- FamilyControls/DeviceActivity/ManagedSettings;
- MatterSupport request path;
- LiveCommunicationKit;
- RealityKit entity ecosystem.

## Band 4 — niche/restricted/compiler/UI-heavy

- CarKey;
- SecureElementCredential;
- LockedCameraCapture extension shell;
- JournalingSuggestions;
- ContactProvider;
- ExtensionFoundation;
- ManagedApp/ManagedAppDistribution;
- MarketplaceKit;
- DockKit;
- RoomPlan;
- TipKit full model;
- TabletopKit.

This banding should drive implementation planning later: solve the smallest ABI primitives needed by Band 2 first and only generalize when Band 3 proves the abstraction reusable.
