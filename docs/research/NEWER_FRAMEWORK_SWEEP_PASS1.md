# Newer Apple Framework Sweep — Native vs Swift Residuals

Research date: 2026-10-07

This pass checks newer/specialized frameworks that could otherwise be incorrectly classified as Swift-only simply because Apple presents them with Swift examples.

## Matrix

| Framework/capability | Finding | Class | Priority |
|---|---|---|---|
| AccessorySetupKit | Core `ASAccessorySession` is Objective-C `NSObject`, queue + event handler | R2/R3 | Native |
| Sensitive Content Analysis | `SCSensitivityAnalyzer` is Objective-C `NSObject` with completion handlers | R2/R3 | Native |
| Image Playground UI | `ImagePlaygroundViewController` is explicitly `@objc`; delegate methods are `@objc` | Hybrid R2/S2 | Native presentation first |
| Image Playground programmatic generation | `ImageCreator` is Swift async/AsyncSequence and is now deprecated in favor of UI surfaces | D | Do not build ABI around deprecated API |
| AlarmKit | Swift class + generic `AlarmConfiguration<Metadata>`, `AlarmMetadata`, async/AsyncSequence | S2 | P1/P2 |
| FamilyControls authorization | `AuthorizationCenter` offers completion-handler authorization surface, but broader framework uses Swift token/value types | Hybrid | Specialized P2 |
| DeviceActivity | Core monitor is a Swift struct with Swift name/schedule/event values | S2 | Specialized P2 |
| ManagedSettings | Swift class but settings/tokens/groups are Swift value/protocol ecosystem; ObservableObject is incidental | S2 | Specialized P2 |

# AccessorySetupKit

Apple exposes:

```objc
@interface ASAccessorySession : NSObject
```

The session activates on a dispatch queue and invokes an event handler.

This is a strong native-path result: modern privacy-preserving accessory discovery/setup does **not** require Swift ABI.

Framework direction:
- bind/use `ASAccessorySession` through objc2/manual public ObjC binding if missing;
- represent event callbacks directly in Rust;
- continue into CoreBluetooth/Network APIs after user selection;
- preserve Apple permission/session semantics.

References:
https://developer.apple.com/documentation/accessorysetupkit/asaccessorysession?language=objc
https://developer.apple.com/documentation/accessorysetupkit

Classification: **R2/R3**.

# Sensitive Content Analysis

Apple exposes:

```objc
@interface SCSensitivityAnalyzer : NSObject
```

with completion-handler APIs such as image analysis.

The Swift async calls in examples are conveniences over a public Objective-C completion surface.

Framework direction:
- use Objective-C completion-handler methods;
- bridge Blocks directly to Rust;
- do not implement Swift async merely for syntax;
- handle the required entitlement separately in compliance/build support.

Reference:
https://developer.apple.com/documentation/sensitivecontentanalysis/scsensitivityanalyzer?language=objc

Classification: **R2/R3**.

This is another example where a modern Apple framework that looks Swift-centric in samples can be eliminated from the Swift ABI backlog.

# Image Playground

## UIKit path is explicitly Objective-C-compatible

Apple now exposes:

```swift
@MainActor @objc @preconcurrency
class ImagePlaygroundViewController
```

The delegate methods are also explicitly `@objc`.

Examples:
- generated-image delegate callback is `@objc`;
- cancellation delegate callback is `@objc`;
- `sourceImage: UIImage?` is `@objc`;
- availability is exposed with an Objective-C selector.

References:
https://developer.apple.com/documentation/imageplayground/imageplaygroundviewcontroller
https://developer.apple.com/documentation/imageplayground/imageplaygroundviewcontroller/delegate-swift.protocol/imageplaygroundviewcontroller(_:didcreateimageat:)
https://developer.apple.com/documentation/imageplayground/imageplaygroundviewcontroller/sourceimage
https://developer.apple.com/documentation/imageplayground/imageplaygroundviewcontroller/isavailable

### Important hybrid detail

Not every configuration property is necessarily Objective-C-compatible. For example, `concepts` is an array of the Swift value type `ImagePlaygroundConcept`.

Therefore the framework should distinguish:
- presentation/lifecycle/delegate: R2;
- some rich configuration values: may require S2 or a simpler supported constructor/bridge.

Do not classify the entire framework as Swift-only.

## Programmatic ImageCreator is a poor ABI target

`ImageCreator` uses:
- async initialization;
- Swift value arrays;
- opaque `some AsyncSequence`.

Apple now marks it deprecated and directs developers to ImagePlaygroundViewController or SwiftUI sheet presentation.

Reference:
https://developer.apple.com/documentation/imageplayground/imagecreator

**Decision:** do not spend Swift-ABI effort implementing a deprecated programmatic path. Prefer the supported UIKit `@objc` view controller.

This is exactly the kind of elimination the research is intended to discover.

# AlarmKit

AlarmKit is a genuine modern Swift residual.

Apple exposes `AlarmManager` as a Swift class, but scheduling is generic:

```swift
func schedule<Metadata>(
    id: Alarm.ID,
    configuration: AlarmManager.AlarmConfiguration<Metadata>
) async throws -> Alarm
```

where configuration is:

```swift
struct AlarmConfiguration<Metadata>
where Metadata : AlarmMetadata
```

Alarm changes and authorization changes are exposed as async sequences.

References:
https://developer.apple.com/documentation/alarmkit/alarmmanager
https://developer.apple.com/documentation/alarmkit/alarmmanager/alarmconfiguration

## Why AlarmKit is not a simple category-A call

A useful alarm configuration may require an application-defined metadata type conforming to `AlarmMetadata`.

Therefore AlarmKit potentially crosses from:
- consuming Apple-defined Swift values
into:
- defining a Rust-backed Swift-conforming metadata type.

That places richer alarm support closer to ActivityKit than Translation.

## Suggested staged support

### Stage 1
Research whether alarms can be scheduled using an Apple-provided/default metadata type.

If so, implement:
- authorization;
- alarm listing;
- cancel/pause/resume/stop;
- minimal schedule.

### Stage 2
Only if custom metadata is required for common use:
- implement `AlarmMetadata` conformance generation;
- generic `AlarmConfiguration<RustMetadata>`.

### Stage 3
AsyncSequence updates.

Classification: **S2, P1/P2** depending on how often applications need prominent alarms.

# FamilyControls

Apple's FamilyControls documentation exposes `AuthorizationCenter` and shows a completion-handler form for authorization.

This suggests the initial authorization operation may be callable without needing Swift async machinery, even though the type is presented as a Swift class.

Reference:
https://developer.apple.com/documentation/familycontrols/authorizationcenter?language=objc

However, the broader framework uses strongly typed Swift selections/tokens such as application/category/domain tokens and integrates with SwiftUI pickers.

Therefore this is **hybrid**, not purely native and not uniformly Swift-only.

## Framework direction

Split support:

1. authorization lifecycle first;
2. token/value representation;
3. selection UI only if required;
4. DeviceActivity/ManagedSettings integration as a specialized package.

The Family Controls entitlement is restricted/approval-based, so this should not shape the general framework core.

# DeviceActivity

Apple exposes:

```swift
struct DeviceActivityCenter
```

with methods such as:

```swift
func startMonitoring(
    DeviceActivityName,
    during: DeviceActivitySchedule,
    events: [DeviceActivityEvent.Name : DeviceActivityEvent]
) throws
```

This is a Swift value-type API using other Swift value types and dictionaries.

Reference:
https://developer.apple.com/documentation/deviceactivity/deviceactivitycenter

Classification: **S2**.

No UI dependency is inherent, but correct support requires Swift value construction and throws/error ABI.

Because DeviceActivity is entitlement/special-purpose functionality, it should follow general Layer-1 Swift value support rather than drive it.

# ManagedSettings

Apple exposes `ManagedSettingsStore` as a Swift class. It contains numerous Swift settings structs/groups and application/category/web-domain token values.

It also conforms to `ObservableObject`, but that does **not** imply the Rust framework needs Combine/ObservableObject for basic configuration.

References:
https://developer.apple.com/documentation/managedsettings/managedsettingsstore
https://developer.apple.com/documentation/managedsettings/managedsettingsstore/appstore

## Important distinction

For writing settings:
- direct property/method ABI support may be enough.

For observing Combine-published effective values:
- Rust can either call ordinary current-value accessors when available;
- or add Combine interop only if an application actually needs the publisher stream.

Do not import the entire Combine abstraction solely because this Apple class conforms to ObservableObject.

Classification: **S2, specialized P2**.

# Cross-pass conclusion

This newer-framework sweep further validates the elimination-first strategy.

Notable APIs eliminated from the Swift backlog:
- AccessorySetupKit;
- Sensitive Content Analysis;
- most Image Playground presentation/delegate behavior.

Notable genuine residuals:
- AlarmKit;
- DeviceActivity/ManagedSettings;
- richer FamilyControls token/selection operations.

The rule remains:

> Classify individual capability paths, not entire frameworks, because Apple increasingly mixes @objc-compatible presentation/controllers with Swift-only value/configuration APIs in the same framework.

That hybrid classification can save substantial ABI work.
