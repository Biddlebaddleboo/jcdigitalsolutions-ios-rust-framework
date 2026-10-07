# Apple API Research

This directory is the durable research record for deciding how much Swift interoperability this framework actually needs.

## Research rule

For each iOS capability, investigate in this order:

1. Pure Rust implementation.
2. Public C/CoreFoundation/Darwin API.
3. Public Objective-C API, preferably through an existing objc2 framework crate.
4. Thin Rust reconstruction of a Swift convenience/overlay on top of the public native API.
5. Only then classify the remaining surface as genuinely Swift-only.

The goal is to avoid building Swift ABI machinery for capabilities Rust can already access at native or near-native cost.

## Cost classes

- **R0 — Pure Rust:** no Apple language ABI needed.
- **R1 — Direct C ABI:** public C/CoreFoundation/Darwin calls; native function-call boundary.
- **R2 — Objective-C / objc2:** native Objective-C message boundary; expected to match ordinary Objective-C call topology.
- **R3 — Rust overlay:** ergonomic Rust layer over R1/R2; must be allocation/copy/dispatch transparent where possible.
- **S1 — Simple Swift ABI:** genuinely Swift-only, but reachable with narrow stable ABI work.
- **S2 — Complex Swift ABI:** generics, resilient values, protocols/existentials, closures, async/AsyncSequence, actor isolation, etc.
- **S3 — Compiler/tooling integration:** macros, generated registration/discovery metadata, result builders, or other compile-time artifacts are part of the contract.
- **D — Defer/avoid:** a lower-level public route exists or the Swift-specific abstraction adds little value to a Rust-first framework.

## Start here

- `RESEARCH_STATUS.md` — compact consolidated conclusions and current architecture implications.
- `MASTER_FRAMEWORK_CENSUS.md` — broad Apple framework-family census and priority queue.

## Native-first capability passes

- `NATIVE_CAPABILITY_MATRIX.md` — first pass across common app capabilities.
- `NATIVE_CAPABILITY_MATRIX_PASS2.md` — security, sensors, background, communications, VPN, device/system services.
- `NEWER_FRAMEWORK_SWEEP_PASS1.md` — newer Apple frameworks, hybrid/native elimination.

## Swift-first residual classification

- `SWIFT_FIRST_RESIDUALS_PASS1.md` — first ranking of genuinely Swift-first APIs.
- `SWIFT_FIRST_RESIDUALS_PASS2.md` — interoperability layers: consume Swift types vs define Rust-backed Swift types vs compiler metadata.
- `HIGH_VALUE_RESIDUAL_AUDIT_PASS2.md` — MusicKit, AdAttributionKit, WorkoutKit, ProximityReader, VisionKit, LiveCommunicationKit, LockedCameraCapture, CarKey, SecureElementCredential, WidgetKit, MatterSupport.
- `SPECIALIZED_RESIDUAL_AUDIT_PASS3.md` — DockKit, RealityKit, TipKit, ContactProvider, ExtensionFoundation, MDM/marketplace and other lower-frequency frameworks.

## Deep feasibility studies

- `STOREKIT2_SWIFT_ABI_FEASIBILITY.md` — first serious runtime ABI target.
- `TRANSLATION_SWIFT_ABI_FEASIBILITY.md` — contained non-UI async Swift API.
- `APP_INTENTS_METADATA_FEASIBILITY.md` — runtime ABI plus compiler/build discovery metadata.
- `MINIMUM_SWIFT_ABI_PRIMITIVES.md` — smallest reusable Layer-1 ABI feature set.

## Overlay / abstraction elimination

- `FOUNDATION_SWIFT_OVERLAY_AUDIT.md` — Foundation Swift value types that bridge to Objective-C/CoreFoundation.
- `SWIFT_LANGUAGE_ABSTRACTIONS_DISPOSITION.md` — Task, AsyncStream, Combine, Observation, Codable, Result and similar Swift application abstractions that should remain Rust-native rather than be reproduced.

## Current architectural conclusion

The common iOS capability path should remain:

```text
Rust application logic
  -> pure Rust when possible
  -> direct public C ABI when available
  -> objc2 / public Objective-C when available
  -> thin Rust ergonomic helper only when useful
  -> optional Swift ABI subsystem only for real capability gaps
```

The research so far strongly indicates that the Swift subsystem can remain optional and relatively narrow.

## Research still open

The repository research is broad but not symbol-exhaustive. Remaining work includes:

- direct current-Xcode SDK `.swiftinterface` inspection;
- machine-readable SDK inventory generation;
- exact symbol/lowering verification for Tier-1 Swift residuals;
- concrete Swift async ABI experiments;
- bridge-cost measurement for Foundation overlay types;
- App Intents build-metadata contract verification;
- lower-frequency framework passes as needed;
- App Store archive/link validation once implementation planning is approved.

Do not begin implementation solely because these research documents exist. The implementation plan remains subject to explicit approval.
