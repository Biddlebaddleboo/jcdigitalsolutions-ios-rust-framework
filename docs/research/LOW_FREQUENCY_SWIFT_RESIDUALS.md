# Low-Frequency Swift Residual Tail Audit

Research date: 2026-10-07

This pass intentionally covers the lower-value end of the Swift-only census.

The purpose is not to promise support. It is to prevent obscure Swift-only frameworks from later being mistaken for foundational iOS requirements.

## Summary

| Framework | Finding | Classification | Default decision |
|---|---|---|---|
| ManagedApp | Modern secure/declarative configuration is Swift-first, but legacy managed app config remains readable through UserDefaults | Hybrid R2/S2 | Support legacy/native path first; modern Swift only for secure/new features |
| ManagedAppDistribution | Swift values/classes/AsyncSequence and SwiftUI presentation; entitlement-restricted enterprise use | S2/P4 | Optional enterprise package |
| MarketplaceKit | Genuine Swift API, but highly policy/region/entitlement-specific | S2/P4 | Optional distribution package |
| ContactProvider | Swift protocol-based app extension | S2/S3/P4 | Only for apps that expose private contact stores system-wide |
| ExtensionFoundation | Swift protocol/generic extension entry model | S2/S3/P4 | Do not make general extension prerequisite |
| RealityKit | ARView is @objc; scene Entity/component model remains Swift-heavy | Hybrid R2/S2 | Native ARKit/Metal/SceneKit first |
| TipKit | UIKit presentation is @objc; Tip model/rules/macros are Swift | Hybrid R2/S2/D | Prefer Rust-owned hint state unless TipKit semantics specifically required |
| CoreHID | Actor + AsyncThrowingStream/value APIs; specialized HID capability | S2/P4 | Optional, platform/use-case driven |
| Assignables | Education/PDF workflow framework; Swift document/view model | D/P4 | PDFKit/Rust domain logic first |
| AutomatedDeviceEnrollment | SwiftUI modal + restricted entitlement | Layer 4/P4 | Do not support early |
| CreateMLComponents | Swift generic ML training/pipeline utility | D | Use Rust/native ML tooling; Core ML for Apple inference |
| DeveloperToolsSupport | Xcode preview/library integration, not shipping app capability | D | No runtime framework support needed |
| TabularData | General data-frame utility | D | Use Rust data-frame/CSV/JSON libraries |
| LightweightCodeRequirements | Swift constraint-builder API over code-signing checks; mainly specialized security/tooling | D/P4 | Security C APIs first; add only concrete gaps |
| Charts | SwiftUI visualization DSL | D | UI-low-priority; UIKit/Metal/custom charting as needed |

---

# ManagedApp: important native fallback

Apple's modern ManagedApp framework provides secure declarative configuration and secrets using Swift types, generic Decodable configuration values and asynchronous sequences.

For example:

```swift
class ManagedAppConfigurationProvider

func configurations<Configuration>(
    Configuration.Type
) async -> some AsyncSequence<Configuration?, Never>
where Configuration : Decodable
```

Reference:
https://developer.apple.com/documentation/managedapp/managedappconfigurationprovider

## Legacy configuration remains native

Apple explicitly documents backward-compatible **legacy app configuration** that uses the MDMv1 `UserDefaults`-based mechanism.

Reference:
https://developer.apple.com/documentation/devicemanagement/configuring-managed-apps-and-extensions

This is significant for the framework:

- basic organization-supplied property-list configuration can remain R2 through Foundation/UserDefaults;
- new declarative configuration and secure secret delivery may require ManagedApp Swift ABI;
- apps should support both when product requirements justify modern ManagedApp.

## Decision

Do not make ManagedApp Swift interop a prerequisite for managed deployment.

Expose:
1. native legacy managed config first;
2. modern ManagedApp as optional S2 enterprise capability.

---

# ManagedAppDistribution

Apple's ManagedAppDistribution framework provides enterprise/education device-management applications with assigned app/package listings, installation state, progress and system-provided presentation.

Reference:
https://developer.apple.com/documentation/managedappdistribution

The framework requires the **Managed App Installation UI** entitlement.

Its data API includes:
- Swift structs such as `ManagedApp`;
- Swift classes such as `ManagedAppLibrary`;
- asynchronous sequences of available apps/packages.

Reference:
https://developer.apple.com/documentation/managedappdistribution/fetching-and-displaying-managed-apps

System-provided presentation types such as `ManagedAppView` are SwiftUI `View` structs.

## Decision

Split:
- non-UI library/install state: S2 category A;
- system-provided managed-app UI: Layer 4 / SwiftUI.

Priority: **P4 enterprise-only**.

---

# MarketplaceKit

MarketplaceKit is a genuine system capability for:
- alternative app marketplaces;
- web distribution;
- alternative-distribution app installation;
- installation-source inspection.

Reference:
https://developer.apple.com/documentation/marketplacekit

Apple requires managed entitlements/approval and behavior varies by supported geographic region.

Reference:
https://developer.apple.com/documentation/marketplacekit/creating-an-alternative-app-marketplace

Apps merely **distributed by** an alternative marketplace do not necessarily need to call MarketplaceKit.

Reference:
https://developer.apple.com/documentation/marketplacekit/distributing-your-app-on-an-alternative-app-marketplace

Core types such as `AppLibrary` are Swift classes and installation/source values are Swift enums/structs.

## Decision

Genuine S2 capability, but product-policy-specific.

Priority: **P4**.

It should never influence the common Rust iOS framework ABI.

---

# ContactProvider

ContactProvider solves a capability distinct from Contacts: making an app-owned contact database available system-wide to Phone, Mail and other Contacts consumers.

Apple's extension must implement:

```swift
protocol ContactProviderExtension :
    ContactItemEnumerating,
    AppExtension
```

Reference:
https://developer.apple.com/documentation/contactprovider/contactproviderextension

This combines:
- Swift protocols;
- extension entry semantics;
- custom enumerator types;
- async invalidation;
- ExtensionFoundation.

## Decision

Ordinary contact read/write remains native Contacts R2.

Only implement ContactProvider for applications that specifically need a system-wide read-only contact-provider domain.

Classification: **S2/S3 P4**.

---

# ExtensionFoundation

Apple's modern generic app-extension model requires a concrete type conforming to:

```swift
protocol AppExtension
```

with an associated `Configuration : AppExtensionConfiguration`.

Reference:
https://developer.apple.com/documentation/extensionfoundation/appextension/

This is a type-definition/compiler/runtime problem, not a simple function call.

## Decision

Do not replace traditional Objective-C-compatible extension points with ExtensionFoundation merely for uniformity.

Support it only where a required Apple extension point specifically adopts this modern model.

Classification: **S2/S3 P4**.

---

# RealityKit

RealityKit is hybrid.

Apple explicitly declares:

```swift
@MainActor @objc @preconcurrency
class ARView
```

Reference:
https://developer.apple.com/documentation/realitykit/arview

So UIKit presentation and some ARView interaction can be reached through Objective-C.

However, the scene model is Swift-heavy:

```swift
class Entity
struct ModelComponent
protocol Component
```

References:
https://developer.apple.com/documentation/realitykit/entity
https://developer.apple.com/documentation/realitykit/modelcomponent
https://developer.apple.com/documentation/realitykit/component

The component set uses generic/protocol existential values and newer system/custom systems use Swift protocol registration.

## Native alternatives

Before adding RealityKit Swift ABI, determine whether the application can use:
- ARKit;
- Metal;
- SceneKit;
- Model I/O;
- Core Animation/UIKit where appropriate.

These already have native Objective-C/C routes.

## Decision

- ARView presentation: **R2**
- core RealityKit entity/component ecosystem: **S2**
- custom RealityKit components/systems: likely **Layer 2**

Priority: **P2-P3 only for AR/spatial products**.

---

# TipKit

TipKit is also hybrid.

UIKit presentation types include:

```swift
@MainActor @objc @preconcurrency
final class TipUIPopoverViewController
```

Reference:
https://developer.apple.com/documentation/tipkit/tipuipopoverviewcontroller

But the actual tip definition normally requires a Swift type conforming to `Tip`, and rules use macro-driven constructs such as:
- `@Parameter`
- `#Rule`

Reference:
https://developer.apple.com/documentation/tipkit/tips/rule

State updates use AsyncStream-style APIs.

Reference:
https://developer.apple.com/documentation/tipkit/tip/shoulddisplay

## Decision

A Rust-first app can implement:
- eligibility;
- display frequency;
- event counts;
- persistence;
- UIKit popovers/custom hint views;

entirely in Rust/native code.

Only implement full TipKit if interoperability with Apple's own TipKit datastore/rules/presentation behavior is specifically valuable.

Default classification: **D/P3**.

---

# Core HID

Apple's CoreHID framework uses modern Swift actors:

```swift
actor HIDDeviceManager
actor HIDDeviceClient
actor HIDVirtualDevice
```

and asynchronous streams such as:

```swift
func monitorNotifications(...) ->
    AsyncThrowingStream<Notification, Error>
```

Reference:
https://developer.apple.com/documentation/corehid

This is a substantial Swift-concurrency surface.

## Decision

Do not add CoreHID to Layer-1 simply to prove actors.

Only research it when a supported target/device use case actually needs HID access or virtual-device creation.

Classification: **S2 P4**.

Platform availability and entitlement/sandbox restrictions must be verified for the exact target before implementation.

---

# Assignables

Assignables provides education-specific PDF assessment/student-work abstractions and SwiftUI editing experiences.

Reference:
https://developer.apple.com/documentation/assignables

The underlying general capabilities are not unique:
- PDF storage/rendering: PDFKit R2;
- annotations/document data: Rust/native domain model;
- collaboration/merge: Rust algorithms/storage.

What is unique is Apple's Assignables interoperability/workflow.

## Decision

Do not implement unless an education product explicitly needs Assignables-compatible documents/workflows.

Classification: **P4**.

---

# AutomatedDeviceEnrollment

Apple provides a SwiftUI modal that lets authorized device administrators add devices to Apple School Manager / Apple Business Manager / Apple Business Essentials.

Reference:
https://developer.apple.com/documentation/automateddeviceenrollment

It requires:
- SwiftUI binding/view integration;
- Bluetooth/camera;
- a restricted Automated Device Enrollment entitlement.

## Decision

This is Layer-4 UI + entitlement-specific functionality.

Priority: **P4**.

It should not affect the core ABI roadmap.

---

# Create ML Components

Create ML Components is a Swift component/pipeline abstraction for building/training ML workflows.

Reference:
https://developer.apple.com/documentation/createmlcomponents

This is not required to execute Core ML models on iOS. Core ML inference is already native-accessible.

Rust applications also have broad options for:
- preprocessing;
- classical ML;
- model training outside device;
- data pipelines.

## Decision

Default **D**.

Only implement Swift ABI for Create ML Components if an application specifically needs Apple's on-device Create ML pipeline implementation.

---

# DeveloperToolsSupport

DeveloperToolsSupport exists to integrate custom SwiftUI views/modifiers and previews into Xcode's design-time library/preview system.

Reference:
https://developer.apple.com/documentation/developertoolssupport

It is not an application runtime capability.

## Decision

**D.**

Do not include it in the shipping framework ABI roadmap.

If the project later wants Xcode-native preview tooling for Rust-authored UI, treat that as independent developer tooling, not runtime framework support.

---

# TabularData

TabularData provides a Swift `DataFrame`, columns, CSV/JSON ingestion and generic column operations.

References:
https://developer.apple.com/documentation/tabulardata/dataframe/rows-swift.property
https://developer.apple.com/documentation/tabulardata/dataframe/init(csvdata:columns:rows:types:options:)

None of this is a unique Apple system capability.

Rust has mature libraries and can implement equivalent tabular processing entirely as R0.

## Decision

**D.**

Never incur Swift ABI just to obtain a data frame.

---

# LightweightCodeRequirements

LightweightCodeRequirements provides Swift value/protocol/builder APIs for expressing code-signing requirements.

Reference:
https://developer.apple.com/documentation/lightweightcoderequirements/launchcoderequirement

The framework itself exposes functions that operate with Security types such as `SecCode`, while Security remains C/native.

Reference:
https://developer.apple.com/documentation/lightweightcoderequirements/seccodecheckvaliditywithprocessrequirement(code:flags:requirement:)

Some behavior is macOS/process-launch oriented; iOS also enforces platform code-signing rules independently.

## Decision

Use Security C APIs first.

Add LightweightCodeRequirements Swift values only if a concrete product needs a requirement expression unavailable through public Security APIs.

Classification: **D/P4**.

---

# Charts

Swift Charts is a SwiftUI visualization DSL.

Apple documents chart construction as composition of chart building blocks in SwiftUI.

Reference:
https://developer.apple.com/documentation/charts/creating-a-chart-using-swift-charts

The framework project's stated UI priorities make this nonessential.

Alternatives include:
- UIKit drawing;
- Core Graphics;
- Core Animation;
- Metal;
- native third-party/Rust chart rendering.

## Decision

**D** for core framework.

Do not build SwiftUI/result-builder interoperability merely for charts.

---

# Tail-census conclusion

By the low-frequency end of the SDK, most remaining Swift-only frameworks fall into one of four buckets:

1. **Policy/entitlement-specific system integrations**
   - MarketplaceKit
   - ManagedAppDistribution
   - AutomatedDeviceEnrollment

2. **Modern extension/compiler models**
   - ContactProvider
   - ExtensionFoundation

3. **High-level convenience/domain frameworks with native substitutes**
   - TipKit
   - CreateMLComponents
   - TabularData
   - Charts

4. **Specialized hardware/spatial stacks**
   - CoreHID
   - RealityKit

None of these should add primitives to the Swift ABI layer until a higher-priority API has already proven that primitive necessary.

The implementation architecture should therefore be optimized around the high-value residual set, not around theoretical completeness across every Swift framework.
