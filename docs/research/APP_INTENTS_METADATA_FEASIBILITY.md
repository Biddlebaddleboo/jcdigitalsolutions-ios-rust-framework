# App Intents: Swift ABI and Build-Metadata Feasibility

Research date: 2026-10-07

## Executive conclusion

App Intents is not merely a Swift function-calling problem.

Apple explicitly documents that, at compile time, metadata describing app intents, app entities, and entity queries is placed into application/framework bundles. At runtime, the system consults that metadata to discover and select executable intent code.

Apple's WWDC material further explains that:

1. the Swift compiler emits information about App Intents types and selected value-level information;
2. another build tool parses that compiler output;
3. the tool generates a `Metadata.appintents` directory in the built product.

Therefore a pure-Rust App Intents implementation needs to solve **two separable problems**:

- a runtime Swift ABI problem: implement/call the public Swift protocol-based API;
- a build-time metadata production problem: emit supported metadata the system can discover.

App Intents should not be used to drive the initial Swift-call layer. It deserves its own tooling workstream after StoreKit/Translation establish basic ABI primitives.

## Public architecture

Apple defines:

```swift
protocol AppIntent :
    PersistentlyIdentifiable,
    _SupportsAppDependencies,
    Sendable
```

An app intent usually appears as a Swift struct conforming to `AppIntent`.

Parameters are declared through compiler-aware syntax such as `@Parameter` / intent parameter property wrappers/macros.

The execution entrypoint is conceptually:

```swift
func perform() async throws -> Self.PerformResult
```

where:

```swift
associatedtype PerformResult : IntentResult
```

This immediately implies:

- protocol conformance metadata/witness tables;
- associated types;
- static protocol requirements;
- Swift value types;
- async/throws;
- result protocols;
- compile-time declarations describing parameters and metadata.

References:
https://developer.apple.com/documentation/appintents/appintent
https://developer.apple.com/documentation/appintents/app-intents
https://developer.apple.com/documentation/appintents/creating-your-first-app-intent

## Compile-time discovery is a hard requirement

Apple explicitly documents:

> At compile time, the compiler places metadata in each bundle about the app intents, app entities, and entity queries it contains.

The system then uses this metadata at runtime when deciding whether/where the intent code can run.

For shared frameworks, Apple provides `AppIntentsPackage` because metadata living only in the framework bundle is otherwise insufficient for normal app/extension discovery.

Reference:
https://developer.apple.com/documentation/appintents/configuring-the-runtime-behavior-of-your-app-intents

This means runtime protocol conformance alone is not enough.

A Rust type that perfectly mimicked the `AppIntent` witness table but did not cause valid bundle metadata to be emitted would not provide normal system discovery.

## Apple's metadata pipeline

Apple's WWDC23 session "Explore enhancements to App Intents" describes the static extraction process:

1. Swift compiler emits information about available types and type/value-level information from App Intents implementations.
2. another tool parses this information;
3. it generates `Metadata.appintents` in the built product;
4. the directory describes intents, parameters, entities, queries, and more.

Reference:
https://developer.apple.com/videos/play/wwdc2023/10103/

This is a stronger requirement than ordinary Swift ABI interoperability.

## Xcode tooling evidence

Public build logs and open-source build-system integrations show the Apple tool:

```text
xcrun appintentsmetadataprocessor
```

Existing Xcode/Bazel integrations invoke it with information such as:

- output path;
- target triple;
- module name;
- toolchain directory;
- compiled binary;
- source/metadata inputs.

Recent build failures also expose intermediate `.swiftconstvalues` artifacts used by the "Extract App Intents Metadata" build phase.

These details are useful research evidence, but they must not be treated as a guaranteed public file-format contract merely because the command exists in Xcode.

Non-authoritative implementation evidence:
https://github.com/bazelbuild/rules_apple/issues/1766
https://github.com/bazelbuild/rules_apple/issues/2760

Apple forum evidence concerning the extraction phase:
https://developer.apple.com/forums/tags/intents

## Important compliance distinction

There are at least three possible approaches, and they are not equally safe.

### A. Produce ordinary Swift ABI/compiler metadata that Apple's own processor consumes

Potentially acceptable if the metadata is part of the supported stable/public compiler ABI and the processor is invoked in its normal Xcode-supported role.

This is the preferred research direction.

### B. Generate `Metadata.appintents` directly by reverse-engineering its current file format

Technically tempting, but **not acceptable as a default framework architecture** without a supported/public contract.

A private or unstable bundle format can change across Xcode/iOS releases and might amount to relying on undocumented platform internals.

### C. Use private runtime registration instead of static metadata

Reject.

Do not call private registration APIs or daemon protocols merely to make system discovery work.

## Research hypothesis

A zero-Swift-source implementation may still be possible if Rust/build tooling emits the same **supported compiler-level ABI artifacts** that a Swift compiler would emit for App Intents declarations, and then lets Apple's standard metadata extraction phase generate the final bundle metadata.

This is materially different from manually fabricating the final private format.

The feasibility question therefore becomes:

> Which compiler-emitted App Intents records are a documented/stable contract, and can a non-Swift frontend emit them in a supported way?

That is the central unknown.

## Runtime problem

Assuming discovery metadata can be solved, the runtime side still requires a Rust-authored type to behave as an App Intent.

Likely requirements include:

- Swift type metadata for a Rust-defined logical type;
- protocol descriptor references;
- conformance descriptors;
- witness tables;
- static properties such as `title`, `description`, `isDiscoverable`, and `supportedModes`;
- associated `PerformResult` metadata and conformance;
- async `perform()`;
- parameter storage/access;
- Sendable-related ABI metadata as applicable;
- runtime initialization/destruction.

This is much more involved than merely calling a StoreKit function.

## Can Rust define a Swift type?

For App Intents, a long-term pure-Rust implementation may require emitting Swift-compatible metadata/conformance records for a type whose executable method bodies are Rust functions or thunks.

The stable Swift ABI documents:

- nominal type metadata;
- type descriptors;
- protocol descriptors;
- conformance descriptors;
- witness tables;
- mangling.

That makes the concept technically grounded.

However, the framework must not assume every compiler-emitted section relevant to App Intents is covered by Swift's general ABI-stability guarantees.

Research must distinguish:

- stable runtime ABI metadata;
- public module-interface contracts;
- App-Intents-specific compiler constant metadata;
- Xcode extraction-tool implementation details.

## Parameter declarations

Apple's normal source model uses intent parameters through Swift compiler-aware declarations.

The Rust framework should eventually expose a Rust-native declaration API, for example conceptually:

```rust
#[app_intent(title = "...")]
struct SearchItems {
    #[intent_parameter(title = "...")]
    query: String,
}
```

but **do not design or implement this macro yet**.

First prove what binary/compiler metadata must result from one minimal Swift intent. The Rust macro should eventually generate Rust/ABI metadata directly, not Swift source.

## Build-time research methodology

A correct investigation should use Apple/Xcode output as an oracle.

### Experiment A1 — minimal Swift reference intent

Outside framework/product source, build a minimal reference application containing exactly:

- one parameterless AppIntent;
- static title;
- `perform` returning a minimal result.

Record:

- compiler invocation;
- generated object/Mach-O sections;
- `.swiftconstvalues` artifacts;
- appintentsmetadataprocessor invocation;
- `Metadata.appintents` output;
- symbol table;
- relevant Swift metadata/conformance records.

The temporary reference is research input, not framework source.

### Experiment A2 — one change at a time

Create binary diffs for:

1. add one String parameter;
2. add optional parameter;
3. add enum parameter;
4. add AppEntity;
5. add query;
6. change title/description;
7. change supported execution mode;
8. add AppShortcut;
9. move type into shared framework/AppIntentsPackage.

This allows mapping input declarations to compiler records without guessing.

### Experiment A3 — processor behavior

Run Apple's normal processor against the reference build while varying:

- binary;
- const-value input;
- source-file arguments if applicable;
- module name;
- target triple.

Determine which pieces are actually required and which are diagnostics/build integration.

### Experiment A4 — stable/public contract audit

For every required record:

- find corresponding Swift ABI documentation or public Apple build documentation;
- mark unsupported/undocumented fields separately;
- do not promote an experiment into shipping architecture until the dependency is judged acceptable under the project's App Store compliance rules.

## Potential intermediate support level

The framework may be able to support **calling existing App Intents** before defining new discoverable intents.

If App Intents exposes public APIs that let application code execute known intent instances/types in-process, those may require only runtime Swift ABI and no compiler metadata production for Rust-defined types.

This should be researched separately because it could provide useful partial interoperability with much lower complexity.

It does not solve exposing Rust application actions to Siri/Shortcuts.

## Metadata extraction and framework packaging

Apple explicitly documents that an App Intent inside a shared framework has special packaging concerns. `AppIntentsPackage` informs the build/system about included package/framework intent types.

For this project, that has two implications:

1. App Intents support cannot live only as a generic reusable Rust binary without considering the consuming app/extension bundle.
2. The eventual build integration may need an app-specific generated metadata/object module even if runtime implementation is shared in the framework.

That generated artifact must still contain no Swift source.

## Likely architecture if feasible

Conceptually:

```text
Rust application declarations
        |
        v
Rust proc macro / build metadata generator
        |
        +--> stable Swift type/conformance metadata
        |
        +--> App-Intents compiler metadata input
        |
        v
normal Xcode App Intents metadata extraction
        |
        v
Metadata.appintents in application/extension bundle

Runtime:
system invokes intent
        |
        v
Swift ABI witness/thunk
        |
        v
Rust perform implementation
```

The build-time and runtime halves should be independently testable.

## Performance implications

If this architecture works, normal intent execution does not require a custom runtime bridge.

After system discovery, a perform invocation can theoretically be:

```text
Apple App Intents runtime
  -> Swift ABI witness thunk
  -> Rust function
```

The thunk should be native-call-scale.

Most complexity is **compile/build metadata complexity**, not hot-path runtime overhead.

This is favorable for the project's performance goals.

## Why a Swift source shim remains undesirable

A conventional workaround would generate/compile a tiny Swift AppIntent struct and forward `perform()` into Rust.

That is technically straightforward, but violates this repository's zero-Swift-source invariant and creates a second declaration model that must be generated/maintained.

The research goal is therefore explicitly stronger: determine whether the necessary metadata and protocol conformances can be emitted without Swift source.

## Risks

### Undocumented metadata risk

App-Intents-specific compile-time extraction data may not be a stable public ABI.

If so, direct generation could be fragile or unacceptable for an App Store-focused framework.

### Toolchain version coupling

Even if Xcode's extraction tool accepts externally generated metadata, its input format may change per Xcode release.

The build layer may need SDK/Xcode-version adapters.

### Macro semantics

`@Parameter` and newer App Intents macros may evolve faster than the core Swift ABI.

### Rich entities/queries

AppEntity, EntityQuery, DynamicOptionsProvider and modern schema integrations can dramatically enlarge the witness/metadata surface.

### Apple Intelligence evolution

App Intent schemas/semantic indexing are actively evolving. Keep the initial target intentionally small.

## Recommended implementation boundary

Do not make "full App Intents" a V1 prerequisite for the core Rust iOS framework.

Instead define staged research/implementation targets:

### Stage 0 — metadata reverse mapping only

No framework implementation. Understand the supported build pipeline.

### Stage 1 — parameterless Rust-defined AppIntent

Success means:

- no Swift source in repository/build inputs;
- discovered in Shortcuts/System;
- public/supported build pipeline only;
- `perform()` reaches Rust;
- installation through an archived/distribution-style build still works.

### Stage 2 — primitive parameters/results

Add:

- String;
- Bool;
- numeric values;
- optional values;
- simple enums.

### Stage 3 — AppEntity/query

Only after basic metadata generation is stable.

### Stage 4 — advanced schemas/Apple Intelligence integrations

Treat independently.

## Distribution validation is mandatory

App Intents has behavior that differs between local Xcode development and installed/distributed builds.

Every milestone must test:

- simulator where supported;
- development-signed physical device;
- archived Release app;
- Ad Hoc/TestFlight-style install if available;
- Shortcuts/Siri/Spotlight discovery after install/update;
- app + extension/shared-framework packaging where relevant.

Do not accept "works when run from Xcode" as sufficient.

## Preliminary conclusion

A pure-Rust App Intents implementation is **plausible but unproven**.

The runtime ABI portion is conceptually covered by Swift's stable type/protocol/witness machinery.

The much larger uncertainty is App Intents' build-time static metadata extraction. Apple explicitly confirms that compile-time metadata is fundamental to discovery, and its build chain generates a `Metadata.appintents` bundle from compiler-produced information.

Therefore:

- classify App Intents as **S3**;
- do not conflate it with StoreKit-style Swift function calls;
- first reverse-map the normal public Xcode extraction pipeline;
- only proceed if required emitted artifacts can be produced through a sufficiently supported/stable interface;
- never fall back to private runtime registration;
- keep Swift source out of the framework even if that means App Intents remains unsupported initially.

### Stage 0 update — 2026-10-07

The [C7 App Intents Stage 0 audit](../swift-abi/APP_INTENTS_STAGE0.md) observed Xcode 26.6 / iOS SDK 26.5 compiling a temporary Swift AppIntent, invoking its normal metadata extraction steps, and producing `Metadata.appintents` in the app bundle. The processor executable, command syntax, generated file lists, and metadata encoding remain observed Xcode implementation details; no documented stable Rust/C metadata input or processor API was found. Combined with C6's no-go for a public C/C++ task-entry contract for async `perform()`, Stage 1 is unsupported on the audited toolchain. The earlier plausibility assessment remains a research hypothesis, not a V1 implementation claim; re-audit on Xcode 27.x.
