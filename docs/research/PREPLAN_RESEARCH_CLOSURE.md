# Pre-Plan Research Closure

Research date: 2026-10-07

## Purpose

This document closes the architecture-level research required before writing the first implementation planning set.

The goal is not to manually enumerate every mangled symbol in one Xcode SDK. The goal is to determine which facts are architectural invariants, which mechanisms are technically viable, and which details should be generated or verified mechanically on the macOS/Xcode execution host.

## Current Apple toolchain baseline

As of 2026-10-07:

- Xcode 27 is released and includes Swift 6.4 and iOS/macOS 27 SDKs.
- Xcode 27.1 RC is available.
- Xcode 27.2 beta 2 is available.
- The implementation host should record the actual selected Xcode build and SDK versions before generating ABI artifacts.

Primary Apple references:

- https://developer.apple.com/documentation/xcode-release-notes/xcode-27-release-notes
- https://developer.apple.com/xcode/system-requirements
- https://developer.apple.com/news/releases/

Do not hard-code research conclusions to one Xcode beta when the installed execution host can provide authoritative current interfaces.

## Conclusion 1: Clang is sufficient for the first Swift ABI backend

Clang explicitly supports:

- `swiftcall`;
- `swiftasynccall`;
- `swift_context`;
- `swift_async_context`;
- `swift_indirect_result`;
- `swift_error_result` where valid for the convention.

This is stronger than the previous assumption that async support might require handwritten assembly for the initial proof.

Clang documentation states that `swiftasynccall` models the low-level Swift async calling convention and guarantees the tail-call form required for compatible async continuation functions on targets where Swift ABI is stable.

Reference:

- https://clang.llvm.org/docs/AttributeReference.html

### Adopted direction

The first Swift ABI backend should be:

```text
Rust
 -> ordinary C ABI
 -> microscopic Clang ABI thunk
 -> public Apple Swift symbol
```

For async:

```text
Rust completion state
 -> C ABI entry
 -> Clang swiftasynccall-compatible thunk
 -> Apple Swift async function
 -> continuation/resume thunk
 -> Rust completion/wake
```

Do not start with handwritten ARM64 assembly.

Assembly remains a later optimization/fallback only if compiler-generated thunks are incorrect or measurably worse.

## Conclusion 2: rustc Swift ABI support is now experimental, not absent

The earlier statement that rustc Swift ABI support was simply unimplemented is outdated.

Current rustc contains a `Swift` ABI variant mapped to LLVM `swiftcc`, and tracking issue rust-lang/rust#156481 records an initial implementation behind:

```text
#![feature(abi_swift)]
```

However, the tracking issue still lists unresolved stabilization questions, including:

- no stable `repr(Swift)` story;
- argument classification questions;
- stability/tiering policy;
- Darwin versus non-Darwin support policy.

The feature is experimental and not suitable as the V1 architectural dependency.

References:

- https://github.com/rust-lang/rust/issues/156481
- https://doc.rust-lang.org/nightly/nightly-rustc/rustc_abi/enum.ExternAbi.html

### Adopted direction

Keep the Swift-call backend replaceable:

```text
framework Swift ABI semantic layer
        |
        +--> V1 Clang thunk backend
        |
        '--> future native rustc extern "Swift" backend
```

Do not make nightly Rust mandatory for V1.

## Conclusion 3: Swift runtime primitives should be reused, not recreated

Swift ABI/runtime documentation continues to support the existing architecture:

- type metadata is the central runtime type handle;
- value witness tables describe layout/copy/move/destroy operations;
- metadata offsets are pointer-width-relative and already account for 32/64-bit platforms;
- Swift runtime provides ownership and metadata machinery;
- Swift stable mangling is specified.

References:

- https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst
- https://github.com/swiftlang/swift/blob/main/docs/ABI/Mangling.rst
- https://github.com/swiftlang/swift/blob/main/docs/Runtime.md
- https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/Metadata.h

### Adopted direction

V1 Swift interoperability consumes Apple's Swift runtime.

Do not implement:

- Swift reference counting;
- a Swift heap;
- generic metadata instantiation logic from scratch when runtime/compiler support exists;
- guessed resilient value layouts.

Use metadata/value witnesses for opaque resilient values.

## Conclusion 4: exact SDK manglings are generated build facts, not architecture constants

Swift stable mangling exists, but exact symbol names for Apple framework declarations vary with the concrete declaration/generic specialization and SDK.

Therefore the repository should not maintain a hand-written permanent table derived from one binary dump.

For every Swift-only capability, the macOS execution host should mechanically record:

1. installed SDK declaration from the corresponding `.swiftinterface`;
2. a minimal reference source compiled with the installed Swift compiler;
3. emitted SIL/LLVM IR/assembly/object undefined references;
4. the resolved public symbol;
5. target triple and deployment version;
6. Xcode build number.

This becomes a generated/test fixture or capability-specific ABI record.

### Planning implication

The first implementation plan does not need to know literal StoreKit manglings.

It needs to name the probe/generation mechanism and acceptance tests.

## Conclusion 5: StoreKit 2 remains the strongest complex Layer-1 proof

Current Apple documentation confirms:

```swift
static func products<Identifiers>(
    for identifiers: Identifiers
) async throws -> [Product]
where Identifiers: Collection, Identifiers.Element == String
```

and purchase includes:

```swift
func purchase(
    confirmIn viewController: UIViewController,
    options: Set<Product.PurchaseOption> = []
) async throws -> Product.PurchaseResult
```

Transaction monitoring exposes:

```swift
static var updates: Transaction.Transactions { get }
```

where `Transaction.Transactions` conforms to `AsyncSequence`.

References:

- https://developer.apple.com/documentation/StoreKit/Product/products(for:)
- https://developer.apple.com/documentation/storekit/product/purchase(confirmIn:options:)-6dj6y
- https://developer.apple.com/documentation/storekit/transaction/updates
- https://developer.apple.com/documentation/storekit/transaction/transactions

### Primitive implications

Product retrieval requires at least:

- Swift String input;
- a concrete Collection/Array-like input lowering;
- async throws;
- array result;
- opaque/resilient Product value handling.

Purchase adds:

- Objective-C `UIViewController` passed through without duplicate wrapper objects;
- Set<Product.PurchaseOption>, although the first proof may use the empty/default set;
- purchase-result enum handling.

Transaction.updates adds concrete AsyncSequence iteration.

### Adopted implementation order

Do not begin Swift ABI work with StoreKit purchase.

Use progressively stronger proofs:

1. scalar/pointer synchronous Swift call;
2. opaque value + metadata/value witness;
3. Swift String;
4. synchronous throwing call;
5. minimal async call;
6. async throws returning opaque value;
7. Translation single-string;
8. StoreKit product retrieval;
9. StoreKit purchase;
10. Transaction.updates concrete sequence.

## Conclusion 6: Translation is the cleanest application-level async proof

Current Translation documentation confirms:

```swift
func translate(String) async throws -> TranslationSession.Response
```

The session also exposes:

- direct non-UI initialization for installed languages;
- `cancel()`;
- Response with source/target `String`;
- batch array APIs;
- a concrete `BatchResponse` AsyncSequence.

References:

- https://developer.apple.com/documentation/translation/translationsession
- https://developer.apple.com/documentation/translation/translationsession/response
- https://developer.apple.com/documentation/translation/translationsession/batchresponse

### Adopted direction

Translation single-string should be the first real Apple-framework proof after the generic ABI primitives.

It exercises:

- Swift class reference;
- String input;
- async throws;
- resilient struct result;
- String extraction;
- cancellation behavior.

It is a better first framework proof than StoreKit because it avoids generic Collection input and purchase-state complexity.

## Conclusion 7: Swift String should be treated as an ABI value, not reimplemented

The project should not depend on private `String` guts such as current small-string/internal object layouts.

Swift String is a standard-library ABI value, but direct internal representation assumptions would be unnecessarily fragile.

The first implementation should use compiler output as the oracle for construction/extraction and isolate String conversion behind a capability-independent adapter.

Possible implementation choices must be benchmarked:

- compiler/thunk-generated construction from UTF-8;
- Foundation/NSString bridging where it is semantically valid;
- stable stdlib entry points revealed by compiler output.

Do not manually encode `_StringGuts`.

References:

- https://github.com/swiftlang/swift/blob/main/docs/ABI/Mangling.rst
- https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

## Conclusion 8: App Intents is a separate compiler/build-metadata subsystem

Apple explicitly documents that app-intent/entity/query metadata is emitted into each bundle at compile time and used by the system at runtime for discovery/execution.

Apple also documents `AppIntentsPackage` as the supported way to associate intent metadata across frameworks/packages and consuming app/extension bundles.

References:

- https://developer.apple.com/documentation/appintents/configuring-the-runtime-behavior-of-your-app-intents
- https://developer.apple.com/documentation/appintents/appintentspackage
- https://developer.apple.com/videos/play/wwdc2025/244/

Xcode's build pipeline uses `appintentsmetadataprocessor` to produce `Metadata.appintents`. The exact tool invocation is toolchain-version-specific and should be observed from Xcode build logs rather than treated as a stable public ABI.

### Adopted direction

App Intents must not be part of the Layer-1 Swift call primitive implementation.

Treat it as:

```text
compiler/build metadata subsystem
 + Swift type/conformance subsystem
 + bundle integration
```

A pure-Rust/no-Swift-source AppIntent remains technically plausible but not sufficiently supported to make full App Intents a V1 core requirement.

V1 may research/prototype Stage 0/1 independently, but failure must not block the rest of the framework.

Never use private runtime registration as a substitute for build metadata.

## Conclusion 9: hybrid frameworks must continue to be classified per symbol

Current Apple docs directly confirm important Objective-C entry points:

- VisionKit `DataScannerViewController`: `@objc`;
- RealityKit `ARView`: `@objc`;
- TipKit `TipUIView`: `@objc`;
- MatterSupport `MatterAddDeviceExtensionRequestHandler`: `@objc`.

References:

- https://developer.apple.com/documentation/visionkit/datascannerviewcontroller
- https://developer.apple.com/documentation/realitykit/arview
- https://developer.apple.com/documentation/tipkit/tipuiview
- https://developer.apple.com/documentation/mattersupport/matteradddeviceextensionrequesthandler

FamilyControls `AuthorizationCenter` exposes older completion-handler functionality alongside Swift async APIs, reinforcing the rule to inspect each operation rather than label the whole framework Swift-only.

WidgetCenter exposes synchronous management calls such as:

- `reloadTimelines(ofKind:)`;
- `reloadAllTimelines()`;

while richer widget/provider definitions remain Swift/protocol/build-system heavy.

References:

- https://developer.apple.com/documentation/familycontrols/authorizationcenter
- https://developer.apple.com/documentation/widgetkit/widgetcenter

### Adopted direction

Capability inventory generation should record per symbol:

- C;
- Objective-C;
- Swift sync;
- Swift async;
- compiler/build metadata;
- entitlement/system-owned.

Do not classify an entire framework with one interop label.

## Conclusion 10: machine-readable SDK inventory belongs in tooling, not hand research

A machine-readable inventory remains useful, but it should be generated from the installed Xcode SDK rather than maintained manually.

The future tooling should parse/index:

- `.swiftinterface`;
- Objective-C headers/modules;
- availability annotations;
- `@objc` exposure;
- async/throws;
- generic signatures;
- actor isolation;
- relevant protocol conformances.

The generated inventory is a build/research artifact and can be refreshed for new Xcode releases.

It is not necessary to manually complete every low-frequency symbol before V1 planning.

## Conclusion 11: archive/link validation is required, but no longer an architecture unknown

The zero-Swift-source design must eventually prove:

- Clang thunk objects link correctly into iOS app/framework targets;
- Swift runtime/stdlib dependencies resolve using supported Apple toolchain/runtime mechanisms;
- no Swift source is generated;
- simulator and device builds work;
- archived Release builds package correctly;
- App Store-compatible public APIs only are used.

This is an implementation validation gate.

It does not require a different architecture.

## macOS Codex execution-host research probes

Before implementing Swift-only capabilities, Codex on macOS should capture an SDK/toolchain manifest:

```text
xcode-select -p
xcodebuild -version
xcrun swiftc --version
xcrun --sdk iphoneos --show-sdk-path
xcrun --sdk iphonesimulator --show-sdk-path
```

For each representative Swift declaration:

1. locate the installed module `.swiftinterface`;
2. preserve the exact public declaration in a generated research fixture;
3. compile a tiny external oracle source using `swiftc`;
4. emit SIL and LLVM IR;
5. inspect object symbols with `nm`/equivalent;
6. inspect optimized assembly;
7. record target triple/deployment target;
8. compare simulator and device lowering when relevant.

These probes use Swift source **only as external research/compiler-oracle input**, not as framework source or generated shipping source.

That distinction preserves the zero-Swift-source framework rule.

## Research that is closed for planning purposes

The following no longer need further conceptual research before the V1 planning set:

- whether most iOS APIs require Swift: they do not;
- whether Clang can express Swift sync/async calling conventions: it can;
- whether rustc native Swift ABI should be mandatory: no, experimental only;
- whether the project should recreate the Swift runtime: no;
- whether resilient Swift values need metadata/value witnesses: yes;
- whether StoreKit is a useful complex forcing function: yes;
- whether Translation is a cleaner first framework async proof: yes;
- whether App Intents belongs in ordinary runtime interop: no;
- whether hybrid frameworks need symbol-level classification: yes;
- whether exact manglings should be manually frozen in architecture docs: no;
- whether physical-device validation remains necessary for iOS performance/hardware claims: yes.

## Remaining implementation-time facts

These are intentionally **not** pre-plan architecture research blockers:

- exact Xcode 27.x installed `.swiftinterface` declaration spellings;
- exact mangled symbols emitted for selected concrete APIs;
- exact SIL/LLVM lowering for each concrete async/generic shape;
- exact archive linker flags automatically selected by the installed Xcode toolchain;
- physical-device performance numbers;
- concrete assembly opportunities;
- per-capability Apple-vs-Rust performance winners.

They must be verified mechanically during execution because they are version/toolchain/hardware dependent.

## Pre-plan verdict

No architecture-level research blocker remains for writing the V1 implementation planning set.

The plan should still begin execution by recording the macOS/Xcode toolchain manifest and generating SDK-specific ABI fixtures before implementing the affected Swift-only workstreams.

That is validation of current toolchain facts, not a reopening of architecture.
