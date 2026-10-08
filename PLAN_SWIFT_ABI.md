# PLAN_SWIFT_ABI.md — Workstream C: Swift ABI and Compiler/Metadata Residuals

## Objective

Implement the smallest reusable zero-Swift-source interoperability layer required for genuinely Swift-only public Apple APIs, then use it for the V1 residual capability set.

The framework must consume Apple's Swift ABI/runtime; it must not build a Swift runtime clone.

## Dependencies

Requires `PLAN_FOUNDATION.md`.
Consumes shared tooling contracts from `PLAN_VALIDATION.md`.

Read first:
- `docs/SWIFT_ABI.md`
- `docs/research/PREPLAN_RESEARCH_CLOSURE.md`
- `docs/research/CLANG_SWIFT_ABI_BACKENDS.md`
- `docs/research/MINIMUM_SWIFT_ABI_PRIMITIVES.md`
- `docs/research/SWIFT_RUNTIME_PRIMITIVES.md`
- StoreKit/Translation/App Intents research docs
- low-frequency residual research

## Write scope

- `interop/swift-abi-core/**`
- `interop/swift-abi-values/**`
- `interop/swift-abi-async/**`
- `interop/swift-abi-generated/**`
- `interop/swift-abi-apple/**`
- `interop/swift-build-metadata/**`
- `tools/swift-oracle/**` only in coordination with G
- Swift-specific ABI fixtures/tests/docs

No `.swift` shipping source.

External tiny Swift source is allowed only as ephemeral/compiler-oracle test input under tooling if it is not framework/application source and is never packaged as shipping source. Prefer generated temporary files from tests/tools so the repository invariant is explicit.

## Execution decomposition

C1 is `PLAN_SWIFT_ABI_OWNERSHIP.md`: end-to-end ownership evidence for the existing `SwiftRetained` wrapper and compiler-derived `swift_retain`/`swift_release` bindings. C1 does not implement general Swift values, direct Apple API calls, or async calls. C2 is [PLAN_SWIFT_ABI_STRING.md](PLAN_SWIFT_ABI_STRING.md): a compiler-authored C++ interoperability proof for a concrete Swift `String` round-trip, using only temporary Swift oracle input. C3 is [PLAN_SWIFT_ABI_OPTIONAL.md](PLAN_SWIFT_ABI_OPTIONAL.md): a compiler-authored C++ interoperability proof for `Optional<String>` with a fixed-width C boundary. C4 is [PLAN_SWIFT_ABI_ASYNC.md](PLAN_SWIFT_ABI_ASYNC.md): a compiler-header feasibility proof for Swift `async throws` functions and a fixed-width C caller path. C5 is [PLAN_SWIFT_ABI_ASYNC_THUNK.md](PLAN_SWIFT_ABI_ASYNC_THUNK.md): a compiler-derived Clang `swiftasynccall` lowering proof after C4's generated-header omission. C6 is [PLAN_SWIFT_ABI_ASYNC_RUNTIME.md](PLAN_SWIFT_ABI_ASYNC_RUNTIME.md): an audit for a supported public Swift task-entry interface and the task/context contract needed by a Rust caller. Each proof is bounded; async runtime machinery and concrete API proofs require separate implementation subplans.

C7 is [PLAN_SWIFT_APP_INTENTS.md](PLAN_SWIFT_APP_INTENTS.md): an isolated Stage 0 audit of the normal public Xcode App Intents metadata pipeline and the supported zero-Swift-source boundary. It records whether the Stage 1 attempt has a documented, stable input path; it does not permit private metadata formats or runtime registration.

## Phase 0 — toolchain manifest and installed SDK inventory

Before writing ABI thunks, record:
- `xcode-select -p`
- `xcodebuild -version`
- `xcrun swiftc --version`
- iphoneos/iphonesimulator SDK paths
- target triples/deployment targets

Generate SDK-specific fixtures by:
- locating public `.swiftinterface`;
- recording exact public declarations;
- compiling minimal compiler-oracle calls;
- emitting SIL/LLVM IR/optimized assembly;
- recording mangled symbols and metadata accessors;
- verifying device/simulator lowering where relevant.

Do not hard-code an ABI signature from memory.

## Phase 1 — synchronous ABI core

### `swift-abi-core`
Proposed internal symbols:
- `SwiftMetadata`
- `SwiftTypeKind`
- `SwiftValueWitnessTable`
- `SwiftValueLayout`
- `SwiftRetained<T>`
- `SwiftError`
- `SwiftSymbol`
- thunk descriptor/version metadata

Use stable Swift runtime C-ABI primitives when verified for the deployment range.

Requirements:
- retain/release native Swift class references using Swift runtime semantics;
- read metadata/value witness information without guessed resilient layouts;
- opaque value state machine: uninitialized -> initialized -> moved/destroyed;
- no memcpy unless witness flags prove bitwise movement/copy is valid;
- alignment and allocation correct on arm64 and simulator.

### Synchronous thunk backend
Preferred order:
1. compiler-generated C++ compatibility wrapper as oracle or direct backend when reliable and representable;
2. Clang `swiftcall` on compiler-derived lowered signature;
3. generated LLVM IR/object if Clang source form is insufficient;
4. handwritten assembly only if required and proven.

Keep thunk backend replaceable so future stable rustc `extern "Swift"` can take over.

## Phase 2 — standard value adapters

### `swift-abi-values`
Implement only concrete primitives required by V1 APIs:

- Swift `String`
- `Optional<T>`
- `Array<T>` for concrete V1 element types
- `Set<T>` only when a concrete API requires it
- frozen/resilient enums through stable compiler/runtime machinery
- concrete generic metadata/witness sets
- Objective-C object pass-through inside Swift calls

Do not implement arbitrary Swift Collection/Protocol/Existential machinery before a consumer requires it.

### String
Do not encode `_StringGuts` manually.
Use compiler/runtime/bridging path proven by oracle output.
Benchmark NSString/Foundation bridging versus compiler-authored direct construction/extraction if both are supported.

## Phase 3 — async/throws

### `swift-abi-async`
Implement concrete compiler-derived async entry/resume support using:
- Clang `swiftasynccall`/`swift_async_context`;
- generated LLVM as fallback;
- Rust completion state from `framework-async`.

Proposed internal concepts:
- `SwiftAsyncContext`
- `SwiftContinuation`
- `SwiftAsyncOperation<T>`
- `SwiftAsyncError`
- cancellation adapter

Required semantics:
- stable lifetime across suspension;
- exactly-once Rust completion;
- error ownership preserved;
- task/executor/actor requirements follow compiler-generated reference behavior;
- no universal Swift-like runtime recreated in Rust;
- no mandatory Rust executor.

## Phase 4 — Translation proof

Implement first real framework proof:
- `TranslationSession` ownership;
- session initialization for supported installed-language path;
- `translate(String) async throws -> Response`;
- source/target String extraction;
- `cancel()`;
- batch support only after single-string path is stable.

Tests:
- compiler-oracle lowering comparison;
- success/error/cancel;
- leak/ownership stress;
- device/simulator availability.

## Phase 5 — StoreKit 2

Implement in increasing complexity:

1. Product property/value access.
2. `Product.products(for:)` with concrete identifiers input.
3. `[Product]` result iteration.
4. purchase using default/empty options first.
5. purchase-result enum handling.
6. verification/transaction value access required for useful purchase handling.
7. concrete `Transaction.updates` AsyncSequence adapter.
8. additional purchase options only after default flow works.

Do not create general Collection/AsyncSequence support merely to satisfy one call. Build concrete adapters, then extract reusable primitives only after a second consumer proves commonality.

Use StoreKit testing/sandbox facilities; never perform unintended real purchases.

## Phase 6 — other Tier-1 residuals

Apply the Layer-1 substrate to current research-prioritized residuals when public availability permits:

- AdAttributionKit
- Foundation Models plain text/session
- WidgetCenter management Swift-only gaps
- selective MusicKit residuals not covered by REST/MediaPlayer
- AlarmKit basic flows if no additional Layer-2 nominal-type machinery is required

Each capability gets its own small module/crate and may not expand common ABI machinery without a concrete need.

## Phase 7 — Layer-2 residuals

Only after Layer 1 is proven, add Rust-defined Swift-compatible types/conformances required by V1-specialized APIs:

Potential consumers:
- ActivityKit
- GroupActivities
- AlarmKit custom metadata
- WorkoutKit
- DeviceActivity/ManagedSettings Swift token/value surfaces
- richer FamilyControls
- MatterSupport request path
- LiveCommunicationKit
- RealityKit entity/component ecosystem
- ProximityReader

Required additions may include:
- nominal type descriptors;
- value witness tables for Rust-backed values;
- protocol conformance descriptors/witness tables;
- associated-type metadata;
- Hashable/Codable-compatible conformances where public ABI permits.

Do not attempt a generic Swift compiler frontend.

If a specific capability requires broad unsupported compiler machinery, classify it as `X` for V1 with an exact documented reason rather than weakening the zero-Swift-source rule.

## Phase 8 — App Intents / compiler metadata

Treat separately from ordinary runtime ABI.

### Stage 0
Observe normal Xcode pipeline:
- compiler-emitted metadata inputs;
- `appintentsmetadataprocessor`;
- bundle `Metadata.appintents`;
- `AppIntentsPackage` relationship.

### Stage 1
Attempt a parameterless Rust-defined AppIntent with:
- no repository/generated shipping Swift source;
- only public/stable Swift ABI/compiler/build mechanisms;
- system discovery;
- `perform()` reaching Rust;
- archived Release install validation.

If the input artifact format required by Xcode is undocumented/toolchain-private and cannot be generated through a supported interface, stop and document App Intents as unsupported for V1. Do not use private runtime registration.

Full WidgetKit provider/rendering, rich AppEntity/query, macro-equivalent Generable, and result-builder/SwiftUI emulation are not allowed to balloon the common Layer-1 runtime.

## ABI safety tests

For every primitive:
- compare generated ABI/signature against compiler oracle;
- simulator + device where required;
- ownership retain/release/destroy stress;
- error paths;
- alignment;
- opaque value initialization/move/destroy;
- async completion/cancellation race;
- no unwinding across thunk;
- public-symbol provenance.

## Performance

Inspect optimized thunk assembly.
Expected incremental synchronous overhead:
- one tiny ABI adaptation call, ideally inline/LTO removable;
- required native ownership only.

Do not introduce C++ heap wrappers or serialization if direct ABI value handling is possible.

If handwritten ARM64/x86-64 assembly is considered:
- retain compiler-generated reference;
- prove parity;
- prove meaningful performance improvement;
- keep narrow fallback.

## Non-goals

- no Swift runtime clone;
- no general Swift parser/compiler;
- no arbitrary SwiftUI;
- no general protocol/existential system before real consumers;
- no nightly-rustc dependency for V1;
- no private symbols;
- no guessed resilient layouts.

## Handoff

Report per residual:
- public API and SDK declaration;
- lowering/oracle fixture;
- thunk backend;
- shared primitives added;
- tests/device status;
- archive/link status;
- unresolved capability-specific compiler metadata.
