# Research Status and Consolidated Findings

Research status date: 2026-10-07

This file is the compact entry point for the Apple API / Swift interoperability research. Read this before opening the deeper research files.

## Core conclusion

The working hypothesis has now been strongly supported:

> Most common iOS capabilities do not require Swift ABI interoperability at all.

The practical capability ladder is:

```text
1. Pure Rust
2. Public C / CoreFoundation / Darwin
3. Objective-C through objc2
4. Thin Rust ergonomic overlay
5. Only then Swift ABI / Swift compiler metadata
```

Swift interoperability should therefore be an **optional residual subsystem**, not the foundation of the framework.

## High-frequency capabilities already eliminated from the Swift problem

The following can be implemented using pure Rust, public C, Objective-C/objc2, or a combination:

- UIKit application lifecycle and ordinary UI;
- Foundation;
- HTTP/networking;
- preferences;
- filesystem;
- Keychain/Security;
- notifications;
- Core Location;
- Core Bluetooth;
- Core Motion;
- background tasks;
- camera capture;
- AVFoundation/AVFAudio playback/capture;
- Photos;
- Contacts;
- EventKit;
- authentication/passkeys;
- LocalAuthentication;
- WebKit;
- CloudKit;
- Core ML;
- Vision;
- Metal;
- MapKit;
- Core Data;
- NFC;
- HealthKit core;
- CallKit;
- PushKit;
- DeviceCheck/App Attest;
- GameKit;
- NetworkExtension/VPN;
- Nearby Interaction;
- HomeKit;
- ExternalAccessory;
- PDFKit;
- Quick Look;
- MessageUI;
- AccessorySetupKit;
- Sensitive Content Analysis;
- much of Image Playground presentation;
- much of VisionKit presentation;
- Apple Pay / PassKit.

For these, Swift syntax may be common in documentation, but Swift is not required to obtain the capability.

## High-value genuine Swift residuals

### Tier 1: implement ABI primitives around these first

1. **StoreKit 2**
   - modern supported IAP path;
   - Swift structs/enums/generics;
   - async/throws;
   - AsyncSequence;
   - strongest first forcing function.

2. **Translation**
   - direct non-UI Swift class;
   - async/throws;
   - Swift String/value result;
   - cleaner second ABI proof.

3. **AdAttributionKit**
   - Swift structs;
   - async static/instance methods;
   - no obvious app-defined Swift protocol type for basic flows.

4. **Foundation Models, plain text/session**
   - Swift class;
   - async/throws;
   - generic response;
   - structured `@Generable` support can be postponed.

5. **WidgetCenter management**
   - small useful Swift-only management API;
   - much smaller target than full widget definition/rendering.

6. **Selective MusicKit**
   - only for capability gaps not already covered by Apple Music REST or MediaPlayer.

### Tier 2: require Rust-defined Swift-compatible types or more complex generic models

- ActivityKit;
- GroupActivities / SharePlay;
- AlarmKit custom metadata;
- WorkoutKit;
- DeviceActivity / ManagedSettings;
- richer FamilyControls;
- MatterSupport request path;
- LiveCommunicationKit;
- RealityKit entity/component ecosystem;
- ProximityReader.

### Tier 3: build/compiler metadata or UI-heavy integration

- App Intents;
- full WidgetKit providers/rendering;
- LockedCameraCapture extension scene;
- ContactProvider;
- ExtensionFoundation;
- structured Foundation Models with macro-equivalent Generable support.

### Tier 4: niche/restricted

- CarKey;
- SecureElementCredential;
- FinanceKit where entitlement unavailable;
- DockKit;
- RoomPlan;
- Journaling Suggestions;
- ManagedApp / ManagedAppDistribution;
- MarketplaceKit;
- full TipKit model;
- TabletopKit.

## Swift ABI feasibility result

Swift ABI stability on Apple platforms makes direct interoperability technically grounded.

However, stable Rust still does not currently provide a mature direct Swift calling convention / `repr(Swift)` solution.

The best first proof path identified is:

```text
Rust logic/state
  -> ordinary C ABI
  -> microscopic Clang swiftcall thunk
  -> documented public Apple Swift API
```

This still satisfies the project's core source rule:

- no Swift source;
- no Swift bridge application layer;
- no serialization runtime;
- no second managed runtime.

The thunk is ABI adaptation only.

Longer-term alternatives:
- generated LLVM IR/object thunks;
- narrow ARM64 assembly thunks;
- future rustc Swift ABI support when mature.

## Swift interoperability layers

### Layer 1 — consume Apple-defined Swift types

Examples:
- StoreKit;
- Translation;
- Foundation Models;
- AdAttributionKit;
- FinanceKit.

Likely primitives:
- Swift symbol/mangling handling;
- calling convention;
- metadata;
- value witnesses;
- strings;
- arrays/sets;
- enums/optionals;
- error ABI;
- async;
- concrete async-sequence adapters.

### Layer 2 — define Rust-backed Swift-compatible types

Examples:
- ActivityKit attributes;
- GroupActivity;
- AlarmMetadata;
- structured generation types.

Adds:
- nominal type descriptors;
- value witness tables;
- protocol conformance descriptors;
- witness tables;
- associated types;
- Hashable/Codable-style conformances.

### Layer 3 — compiler/build metadata

Primary example:
- App Intents.

Adds:
- compiler-like static metadata;
- Xcode extraction-tool integration;
- bundle registration/discovery;
- distribution validation.

### Layer 4 — declarative UI / DSL compatibility

Examples:
- arbitrary SwiftUI;
- full WidgetKit rendering;
- result-builder-heavy APIs.

This is intentionally lowest priority.

## Important hybrid-framework findings

Do not label frameworks wholesale.

Examples discovered:

- **VisionKit:** `DataScannerViewController` is explicitly `@objc`, while some config values are Swift.
- **RealityKit:** `ARView` is explicitly `@objc`, while Entity/component APIs are Swift.
- **TipKit:** `TipUIView` is `@objc`, while Tip definitions/rules are Swift.
- **MatterSupport:** extension request handler is `@objc`, request execution is Swift async/value based.
- **Image Playground:** view controller/delegate is `@objc`; some configuration is Swift.
- **FamilyControls:** some authorization path is simpler than its broader Swift token/picker ecosystem.
- **LockedCameraCapture:** capture itself stays AVFoundation/UIKit; the special extension shell is Swift-heavy.
- **MusicKit:** Apple Music REST and MediaPlayer eliminate large portions of the Swift requirement.

The unit of research must remain the **capability/symbol**, not merely the framework.

## Performance conclusion

For native-first capabilities:

```text
Rust -> C ABI
```

or

```text
Rust -> objc2 -> objc_msgSend
```

can be essentially at ordinary native API cost.

For Swift-only capabilities, the target topology is:

```text
Rust -> microscopic ABI thunk -> Apple Swift API
```

not:

```text
Rust -> serialization -> bridge runtime -> Swift app layer
```

For compute-heavy work, keep data and algorithms in Rust and cross Apple language boundaries only for system capabilities.

## Research documents

Read in this order:

1. `MASTER_FRAMEWORK_CENSUS.md`
2. `NATIVE_CAPABILITY_MATRIX.md`
3. `NATIVE_CAPABILITY_MATRIX_PASS2.md`
4. `SWIFT_FIRST_RESIDUALS_PASS1.md`
5. `SWIFT_FIRST_RESIDUALS_PASS2.md`
6. `STOREKIT2_SWIFT_ABI_FEASIBILITY.md`
7. `TRANSLATION_SWIFT_ABI_FEASIBILITY.md`
8. `APP_INTENTS_METADATA_FEASIBILITY.md`
9. `NEWER_FRAMEWORK_SWEEP_PASS1.md`
10. `HIGH_VALUE_RESIDUAL_AUDIT_PASS2.md`
11. `SPECIALIZED_RESIDUAL_AUDIT_PASS3.md`

## Remaining research before implementation planning

The research is broad enough to define architecture, but not yet symbol-exhaustive.

Remaining useful work:

- inspect installed current Xcode SDK `.swiftinterface` files directly;
- produce a machine-readable symbol/capability inventory from SDK interfaces;
- verify exact StoreKit 2 public ABI declarations/manglings;
- use compiler output as an ABI oracle for representative call shapes;
- map Swift standard-library construction/destruction primitives needed by Layer 1;
- investigate Apple Swift async entry/resume ABI in enough detail for a prototype;
- verify symbol-level Objective-C exposure for important hybrid frameworks;
- continue low-frequency framework census as new Apple SDKs add frameworks;
- verify App Intents build metadata contract versus Xcode implementation detail;
- evaluate App Store archive/link behavior for a zero-Swift-source swiftcall proof.

## What should not happen yet

Do not:

- build a generic Swift runtime clone;
- build arbitrary SwiftUI support;
- implement every Swift protocol mechanism before a concrete API needs it;
- add Swift source;
- generate Swift source;
- use private symbols to make an unsupported API callable;
- start StoreKit implementation before the implementation plan is explicitly approved.

The current work remains research/documentation only.
