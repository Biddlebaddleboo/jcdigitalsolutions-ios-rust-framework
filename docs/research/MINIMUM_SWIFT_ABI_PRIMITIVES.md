# Minimum Swift ABI Primitive Set

Research date: 2026-10-07

This document reduces the Swift interoperability problem to the minimum reusable primitives required by the highest-priority residual frameworks.

It is intentionally not a general Swift runtime design.

## Scope driver

Tier-1 concrete consumers:

- StoreKit 2
- Translation
- AdAttributionKit
- Foundation Models plain-text/session APIs
- WidgetCenter management
- selective MusicKit APIs

The ABI layer should add no primitive that lacks a concrete consumer among these APIs unless correctness forces it.

# 1. Native Swift call boundary

## Requirement

Rust needs a way to invoke a public Swift function/method using Swift's calling convention.

Swift's stable ABI documents:

- ordinary argument/result registers;
- indirect result register;
- Swift self/context register;
- Swift error register;
- Swift async context register.

On arm64:

- x0-x7: normal integer args/results;
- x8: indirect result;
- x20: Swift context/self;
- x21: error result;
- x22: async context;
- v0-v7: floating/SIMD args/results;
- x18: reserved.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst

## Design consequence

A Swift call is not safely expressible as a normal `extern "C"` declaration when Swift-specific lowering/registers are involved.

The current first implementation candidate remains:

```text
Rust
 -> standard C ABI
 -> tiny Clang swiftcall thunk
 -> public Swift ABI symbol
```

The thunk must contain only ABI adaptation.

# 2. Symbol identity / mangling

Swift stable mangling is documented and public symbols use the stable `$s` scheme.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/Mangling.rst

## Minimum need

The framework must be able to identify/link exact public ABI symbols corresponding to SDK `.swiftinterface` declarations.

Do not hard-code a symbol merely from one OS binary without mapping it back to a public declaration.

## Initial approach

For first prototypes:
- inspect Xcode SDK `.swiftinterface`;
- compile an isolated reference call as an oracle;
- record the emitted symbol;
- verify it corresponds to the public interface.

A general mangling implementation is not a prerequisite for the first ABI proof.

# 3. Type metadata

Swift keeps metadata for every concrete type, including generic instantiations.

The metadata record points to a value witness table and identifies the kind of type.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

## Minimum uses

Needed for:
- resilient StoreKit/Translation values;
- generic `VerificationResult<Transaction>`;
- `Array<Product>`;
- `Response<String>`;
- metadata arguments to generic functions.

## Rule

Do not hard-code resilient type sizes/layouts.

Resolve/use metadata and value witnesses.

# 4. Value witness operations

A value witness table provides operations/properties for Swift values, including:

- size;
- alignment;
- stride;
- initialize/copy;
- move;
- destroy;
- enum payload/discriminator behavior where present.

References:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst
https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/Metadata.h

## Minimum wrapper concept

Internal Rust API should eventually be able to represent:

```text
SwiftTypeMetadata
SwiftValueLayout
OpaqueSwiftValue
```

with operations conceptually:

- allocate correctly aligned storage;
- initialize;
- borrow;
- copy only when required;
- move;
- destroy.

Do not expose this directly to app code.

# 5. Swift classes

On Apple platforms, Swift class metadata fundamentally interoperates with the Objective-C runtime layout at the class-object level.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

This does **not** imply Swift-only methods are Objective-C selectors.

## Minimum requirement

Need correct:
- class reference ownership;
- class instance method calls via Swift ABI;
- Swift retain/release semantics where required;
- conversion/pass-through of Objective-C objects accepted by Swift methods.

Consumers:
- TranslationSession;
- LanguageModelSession;
- WidgetCenter;
- Music players;
- FinanceStore;
- ConversationManager.

# 6. Swift String

Many Tier-1 APIs require Swift `String`.

Examples:
- Translation input/output;
- StoreKit identifiers/text properties;
- AdAttributionKit JWS input;
- Foundation Models prompts/results.

## Constraint

Do not guess String's internal layout.

Swift standard-library ABI is part of Apple's stable ABI, but construction/access should rely on documented/stable entrypoints or compiler-generated ABI behavior rather than private layout assumptions.

## Research goals

Find the smallest public/stable path for:
- UTF-8/UTF-16 -> Swift String;
- borrowing/extracting UTF-8 where possible;
- destruction;
- avoiding duplicate conversion.

A single unavoidable string conversion is acceptable; hidden repeated conversion is not.

# 7. Swift Array

StoreKit product lookup and many response APIs use Arrays.

Minimum support:
- construct `[String]` or otherwise satisfy product lookup input;
- consume `[Product]`;
- read count;
- access elements;
- destroy.

## Preferred first strategy

Do not implement arbitrary Collection first.

Inspect whether the concrete public StoreKit call can be made with Array and concrete metadata/witnesses using compiler lowering as an oracle.

Then extract only the reusable Array primitives.

# 8. Swift Set

StoreKit purchase options use `Set<Product.PurchaseOption>`.

This is lower priority than Array.

For first purchase tests:
- determine whether empty/default option sets avoid constructing complex values;
- add Set construction only when required for useful options.

Do not build generic Set support before a concrete StoreKit consumer demands it.

# 9. Enums and optionals

Tier-1 APIs use:
- optional values;
- purchase-result enums;
- VerificationResult;
- error enums;
- state/status enums.

## Two cases

### Frozen enum

Layout/discriminator may be ABI-stable, but still prefer value witnesses/metadata unless direct layout materially helps.

### Resilient enum

Treat as opaque and use public accessors/value witnesses.

## Rule

Do not switch on raw memory tags unless stable ABI documentation guarantees that representation for that concrete type.

# 10. Generic metadata and witness tables

Swift generic ABI passes type metadata and required protocol witness tables.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

Tier-1 consumers may require:
- `VerificationResult<Transaction>`;
- `Response<String>`;
- generic Product lookup collection;
- async sequence iterator types.

## Strategy

Begin with **concrete generic instantiations**, not general generic invocation.

Resolve the exact metadata/witnesses for the concrete types used by each API.

Generalize only when multiple real APIs need the same mechanism.

# 11. Throwing functions

Swift's calling-convention documentation specifies a dedicated error register on supported architectures; on arm64 it is x21.

A throwing call reports either:
- null/zero error register on success;
- error object pointer on failure.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

## Minimum Rust abstraction

Internal result adapter should:
- check the Swift error result immediately;
- retain/own error correctly;
- preserve native error information;
- optionally bridge NSError-compatible errors where available;
- never discard domain/code/context merely to produce a string.

# 12. Async functions

Swift async functions have an ABI distinct from synchronous functions.

Swift's calling-convention summary reserves an async-context register:
- x22 on arm64.

Swift Evolution explicitly notes that async functions use an incompatible calling convention relative to synchronous functions.

References:
https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst
https://github.com/swiftlang/swift-evolution/blob/main/proposals/0296-async-await.md

## Critical caution

The high-level fact that x22 is the async context is **not sufficient** to hand-author arbitrary async entry/resume calls correctly.

The exact lowering includes:
- async context layout;
- continuation/resume functions;
- task/executor interaction;
- result/error placement;
- potential actor isolation.

## First implementation rule

Use compiler output as the oracle.

For each representative shape:
1. compile isolated reference Swift outside framework source;
2. inspect SIL/LLVM IR/assembly;
3. reproduce only the stable/public ABI contract;
4. verify on device and simulator.

Do not invent an async ABI from register summaries.

# 13. Rust async integration

The project should not create a second Swift-like task runtime.

Target:

```text
Rust Future/callback state
 -> ABI thunk
 -> Apple's Swift async runtime
 -> completion/resume thunk
 -> Rust wake/result
```

Potential Rust representations:
- callback-first internal primitive;
- optional `Future` adapter;
- explicit cancellation token if the Apple API exposes cancellation semantics.

Avoid a global executor requirement.

# 14. AsyncSequence

StoreKit transaction updates and some other frameworks use concrete AsyncSequence types.

## Rule

Do not implement arbitrary protocol-based AsyncSequence first.

Implement a concrete iterator adapter for:
1. StoreKit `Transaction.updates`;
2. only then a second concrete sequence if needed.

If both share reusable iteration primitives, extract them.

This avoids prematurely implementing the full Swift protocol/existential/generic surface.

# 15. Objective-C object arguments inside Swift calls

Some Swift APIs accept UIKit/Foundation Objective-C-compatible objects.

Example:
- StoreKit purchase confirmation with `UIViewController`.

These objects already exist as objc2 references.

## Goal

Pass the same native object pointer/reference through the Swift call without wrapping it in a duplicate object.

This is a key zero-copy/zero-extra-object invariant.

# 16. Runtime calls use standard calling convention

Swift's ABI stability documentation states that the Swift runtime uses the platform standard calling convention, even though Swift-to-Swift functions use the Swift convention.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

## Architectural benefit

Rust can call suitable exported runtime helpers through ordinary C ABI, while only actual Swift method/function calls require special thunks.

This should keep the amount of `swiftcall` code very small.

# 17. Memory-management boundary

Swift runtime functionality covers allocation/reference counting/type operations.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/Runtime.md

## Rule

Do not implement Swift reference counting in Rust.

Use the platform Swift runtime's supported ABI operations where required.

The framework only tracks ownership state and invokes the native runtime correctly.

# 18. Rust compiler status

Rust currently tracks Swift function-call ABI support under `#![feature(abi_swift)]`, but as of this research date the tracking issue is marked unimplemented and requires an RFC.

Reference:
https://github.com/rust-lang/rust/issues/156481

## Consequence

Do not make initial implementation depend on unstable/unimplemented rustc Swift ABI support.

Design the internal call boundary so a future Rust-native implementation can replace the thunk backend.

# Minimum Layer-1 implementation order

The research suggests this sequence:

## Phase A — synchronous foundations

1. symbol linking;
2. Swift-call thunk for scalar/pointer-only function;
3. class reference ownership;
4. type metadata lookup;
5. value witness read/use;
6. opaque value allocation/destruction;
7. Swift String;
8. synchronous throwing call.

## Phase B — concrete containers/values

9. Array<String>;
10. consume Array<opaque Apple Swift value>;
11. optionals/enums;
12. concrete generic metadata;
13. Set only if StoreKit options require it.

## Phase C — async

14. one minimal public async call;
15. throwing async call;
16. return opaque value;
17. Rust callback/Future wake;
18. cancellation semantics.

## Phase D — framework proofs

19. Translation single-string call;
20. StoreKit product retrieval;
21. StoreKit purchase;
22. AdAttributionKit simple async struct method;
23. Foundation Models plain response.

## Phase E — streaming

24. StoreKit Transaction.updates concrete iterator;
25. second AsyncSequence consumer;
26. extract reusable sequence primitive if warranted.

# What Layer 1 explicitly does not require

Do not include yet:

- Rust-defined Swift nominal types;
- protocol conformance emission;
- existential construction in general;
- associated type metadata for Rust-defined protocols;
- Codable generation;
- Hashable generation;
- App Intents metadata;
- result builders;
- SwiftUI View;
- actors defined in Rust;
- general Swift macro emulation;
- generic Swift compiler frontend.

Those belong to later layers only if concrete capabilities require them.

# Performance target

For a synchronous Swift API with already-constructed arguments:

```text
Rust call
 -> one ABI thunk
 -> Swift public API
```

The incremental overhead goal is approximately:
- one native call boundary;
- minimal register reshuffling;
- only ownership operations required by Swift semantics.

For async APIs, Apple runtime/task overhead is intrinsic to the public API. Framework-added overhead should be limited to:
- ABI thunk;
- one compact Rust completion state;
- wake/callback.

No serialization, IPC, interpreter, GC, or duplicate object model should be introduced.

# Validation requirement

Every primitive must be validated by:
- optimized assembly inspection;
- ownership stress tests;
- leak checks;
- device and simulator;
- OS-version availability;
- public-symbol provenance;
- App Store-compatible archive/link test.

A primitive is not considered stable merely because one experimental call succeeds.
