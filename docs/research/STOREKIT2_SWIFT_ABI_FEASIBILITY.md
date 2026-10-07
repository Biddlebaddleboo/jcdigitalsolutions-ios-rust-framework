# StoreKit 2 / Swift ABI Feasibility

Research date: 2026-10-07

## Why StoreKit 2 is the first serious Swift-ABI target

StoreKit 2 is a strong forcing function for direct Swift interoperability because the modern in-app purchase API is built around Swift value types, generics, async functions, and asynchronous sequences.

Unlike many Apple frameworks, this is not merely a Swift convenience layer over an equally current Objective-C API. The older `SKPaymentQueue` path is deprecated, so a Rust-first framework needs a credible StoreKit 2 strategy rather than permanently falling back to Objective-C StoreKit 1.

This makes StoreKit 2 a useful scope boundary: implement exactly the Swift ABI features needed for real purchases before considering a more general Swift interoperability layer.

## Relevant StoreKit 2 API shapes

### Product

Apple exposes:

```swift
struct Product
```

Product retrieval is:

```swift
static func products<Identifiers>(
    for identifiers: Identifiers
) async throws -> [Product]
where Identifiers: Collection, Identifiers.Element == String
```

This immediately introduces:

- Swift value types;
- generic function parameters;
- protocol-conformance witness arguments for `Collection`;
- Swift `String`;
- Swift `Array<Product>`;
- async;
- throwing;
- ownership/destruction for returned Swift values.

Reference:
https://developer.apple.com/documentation/storekit/product
https://developer.apple.com/documentation/storekit/product/products(for:)

### Purchasing

Apple exposes purchase entrypoints returning:

```swift
Product.PurchaseResult
```

with methods such as:

```swift
func purchase(
    confirmIn: UIViewController,
    options: Set<Product.PurchaseOption>
) async throws -> Product.PurchaseResult
```

A successful purchase contains a `VerificationResult<Transaction>`.

This adds:

- instance method calling;
- a Swift value-type receiver;
- Swift `Set`;
- nested Swift structs/enums;
- an Objective-C object argument (`UIViewController`) inside a Swift-call ABI;
- async/throws;
- associated-value enums.

References:
https://developer.apple.com/documentation/storekit/product/purchase(confirmIn:options:)
https://developer.apple.com/documentation/storekit/product/purchaseresult
https://developer.apple.com/documentation/storekit/verificationresult

### VerificationResult

Apple declares:

```swift
@frozen
enum VerificationResult<SignedType>
```

The `@frozen` attribute is helpful because its ABI layout is intended to remain fixed rather than resilient, but it is still a generic associated-value enum and therefore requires concrete type metadata/value witnesses to manipulate correctly.

Reference:
https://developer.apple.com/documentation/storekit/verificationresult

### Transactions

Apple exposes:

```swift
static var updates: Transaction.Transactions { get }
```

where the returned type conforms to `AsyncSequence`.

Finishing a transaction is:

```swift
func finish() async
```

A complete StoreKit implementation therefore eventually needs either direct AsyncSequence interoperability or a narrower mechanism that invokes the concrete iterator implementation and bridges yielded elements into Rust.

References:
https://developer.apple.com/documentation/storekit/transaction/updates
https://developer.apple.com/documentation/storekit/transaction/finish()

## Swift ABI stability makes this technically grounded

Swift 5's ABI is stable on Apple platforms. Apple operating systems ship the Swift runtime and standard library as OS components, analogous in deployment terms to the Objective-C runtime.

This matters for the framework design:

- direct Swift ABI interoperability does not inherently require bundling a second language runtime;
- StoreKit itself already relies on the platform Swift runtime;
- a Rust caller can theoretically participate in the same stable ABI rather than serializing through a custom bridge;
- the performance goal can therefore be "direct ABI thunk plus intrinsic Swift/StoreKit costs," not "Rust -> bridge process/runtime -> Swift."

References:
https://www.swift.org/blog/abi-stability-and-more/
https://www.swift.org/blog/abi-stability-and-apple/

## Public ABI documentation exists

Swift's open-source repository documents major pieces of the stable ABI:

- symbol mangling;
- calling conventions;
- type layout;
- type metadata;
- generic metadata;
- value witness tables;
- protocol witness tables;
- existential containers.

References:
https://github.com/swiftlang/swift/blob/main/docs/ABI/Mangling.rst
https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeLayout.rst
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

This is materially better than reverse-engineering an undocumented private interface. The remaining challenge is generating ABI-correct calls from Rust and restricting use to public Apple API symbols.

## ARM64 Swift calling convention

Swift uses a dedicated calling convention for Swift-to-Swift calls.

On arm64, the documented convention includes the ordinary argument/result register set plus Swift-specific registers. Important entries include:

- `x0-x7` — normal integer arguments/results;
- `x8` — indirect result location where required;
- `x20` — Swift context / `self`;
- `x21` — Swift error return;
- `x22` — Swift async context;
- `x18` — platform-reserved; must not be used;
- `v0-v7` — floating-point/SIMD argument/result registers.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/CallingConventionSummary.rst

The important point is that this is more than simply declaring an `extern "C"` function with a mangled name.

## Swift runtime calls versus Swift function calls

Swift's ABI-stability documentation distinguishes the Swift function calling convention from the runtime interface. The Swift runtime itself predominantly uses the platform's standard calling convention.

That creates a promising architecture:

- call ABI/runtime metadata helpers through their standard/C-style ABI where supported;
- use narrowly scoped Swift-call thunks only for functions that actually require the Swift calling convention;
- avoid writing a second runtime.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

## Type metadata and value witnesses are central

Every concrete Swift type has metadata. Generic instantiations may have metadata created lazily by the runtime.

A value witness table provides fundamental operations such as:

- size;
- alignment;
- stride;
- initialize/copy;
- move;
- destroy;
- enum payload/discriminator operations where applicable.

Generic metadata records also carry type-argument metadata and required protocol witness tables.

This means a robust Rust implementation should not assume the in-memory size/layout of resilient StoreKit structs. It should manipulate them using metadata/value witnesses where the ABI requires opacity.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

## Existential and protocol representation is documented

The stable ABI documents opaque existential containers as conceptually containing:

- a three-pointer inline value buffer;
- type metadata;
- one or more protocol witness-table pointers.

Class-constrained existentials use a smaller representation centered on the object pointer plus witness tables.

This is relevant for future protocol-heavy APIs, although StoreKit should initially avoid general existential support where concrete generic types can be used instead.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeLayout.rst

## Critical current blocker: stable Rust cannot directly express Swift ABI

As of 2026-10-07, Rust does not have stable Swift calling-convention support.

Rust issue #156481 tracks `#![feature(abi_swift)]`. The issue was opened on 2026-05-12 and is marked experimental, needs an RFC, and currently unimplemented.

Its unresolved questions also include representation concerns such as the lack of a mature `repr(Swift)` model.

This is the most important feasibility finding from this pass:

> Swift ABI stability exists, but stable Rust currently cannot simply declare StoreKit symbols as `extern "swift"` and rely on rustc to perform all ABI lowering.

Reference:
https://github.com/rust-lang/rust/issues/156481

## Candidate implementation strategies

### Strategy A — microscopic Clang `swiftcall` thunks

Clang publicly supports the `swiftcall` attribute.

Clang's documentation explicitly says `swiftcall` invokes the Swift calling convention and supports attributes such as `swift_indirect_result` for representing the high-level ABI lowering.

Possible architecture:

```text
Rust
  -> C/C++ ABI helper
  -> tiny Clang swiftcall thunk
  -> public StoreKit Swift symbol
```

This would contain **zero Swift source** and no custom bridge runtime.

The shim must remain microscopic: ABI adaptation only. It must not become an application/business-logic layer.

Advantages:

- compiler performs arm64 register/call-convention lowering;
- likely easiest first executable proof;
- ordinary Rust-to-C call plus one Swift thunk should be extremely small overhead;
- no serialization or copied object model is required.

Disadvantages:

- still adds C/C++ source/build tooling;
- the developer must accurately model Swift's high-level lowering before `swiftcall`;
- generic metadata/witness arguments remain our responsibility;
- resilient Swift values remain nontrivial.

Reference:
https://clang.llvm.org/docs/AttributeReference.html

### Strategy B — Rust + hand-written arm64 assembly thunks

For fixed concrete call shapes, assembly can place Swift-specific context/error/async registers directly.

Advantages:

- no C/C++ intermediary;
- exact control;
- potentially minimal instruction overhead.

Disadvantages:

- high correctness burden;
- every complex ABI shape must be lowered correctly;
- simulator/device architecture differences;
- generic/value-return handling remains difficult;
- async ABI is substantially more complex than an ordinary synchronous call.

Recommendation: do not start here. Use only after a Clang/LLVM proof establishes the correct ABI shape and generated assembly can be used as a reference.

### Strategy C — generated LLVM IR/object thunks

LLVM supports Swift calling conventions internally. A build tool could generate LLVM IR or native object files rather than Swift source.

Advantages:

- no Swift source;
- can encode ABI conventions directly;
- potentially keeps the ABI adapter generated and narrow.

Disadvantages:

- build-system complexity;
- LLVM version/toolchain compatibility;
- harder diagnostics;
- must validate Xcode/App Store archive behavior.

This is worth researching after the initial feasibility proof, especially if the goal becomes eliminating C/C++ source as well.

### Strategy D — future rustc `abi_swift`

If Rust's `abi_swift` experiment becomes implemented and eventually stable, migrate suitable thunks to direct Rust declarations.

The framework should design its internal Swift-call boundary so the implementation can be swapped later without changing higher-level APIs.

Do not make the current framework depend on an unimplemented unstable feature.

## Performance expectation

A well-designed ABI adapter should not resemble JNI, React Native, or a serialized bridge.

The desired topology is:

```text
Rust call
  -> tiny ABI thunk
  -> StoreKit Swift entrypoint
  -> Apple's Swift runtime / StoreKit
```

Potential incremental cost can be limited to:

- one native function call into the thunk;
- argument reshuffling required by the Swift ABI;
- metadata/witness lookup required by the actual Swift generic API;
- Rust callback/Future state bookkeeping for async completion.

There should be no requirement for:

- JSON;
- IPC;
- duplicate models;
- global interpreter/runtime;
- arbitrary heap boxing of every argument;
- data serialization solely because the caller is Rust.

The most significant costs for many StoreKit operations will still be StoreKit/UI/network/system work, not the thunk.

## Avoiding unnecessary generic ABI complexity

The generic signature of `Product.products(for:)` is a warning.

Calling the fully generic function may require:

- a concrete collection value;
- collection type metadata;
- a `Collection` witness table;
- knowledge that the element type is Swift `String`.

Before implementing generic invocation machinery, inspect the installed StoreKit `.swiftinterface`, emitted symbols and compiler lowering to determine whether there is:

- an ABI-public specialized entrypoint for common collection types;
- a non-generic internal/public helper exposed through another supported public form;
- a practical way to construct the concrete generic call using standard-library metadata/witnesses.

Do not depend on non-public specializations merely because they happen to exist in one OS release.

## Swift String / Array / Set policy

Do not manually hard-code layouts for resilient/complex standard-library values unless the stable ABI explicitly guarantees them.

Preferred direction:

- obtain public/stable type metadata;
- allocate value storage according to value witness size/alignment;
- initialize/copy/destroy through stable ABI/runtime operations where applicable;
- use public ABI entrypoints for value construction;
- avoid unnecessary Rust <-> Swift string copying, but correctness comes first.

The same rule applies to `Array<Product>`, `Set<PurchaseOption>`, and other standard-library containers.

## Async strategy

Do not implement a second async runtime.

Apple's Swift concurrency runtime already executes StoreKit async operations.

The Rust side should aim to provide:

```text
Rust request/Future state
   -> Swift async entry thunk
   -> Apple Swift concurrency runtime
   -> Swift async completion
   -> tiny continuation/resume thunk
   -> wake/complete Rust state
```

Open questions:

- exact async entry/resume ABI for the targeted SDK;
- lifetime and ownership of the async context;
- error propagation through the Swift error ABI;
- cancellation mapping;
- MainActor requirements for purchase UI;
- whether Rust Future cancellation can/should cancel the underlying Swift Task.

## AsyncSequence strategy

Do not build general `AsyncSequence` support first.

For `Transaction.updates` and `Transaction.unfinished`, first inspect the concrete `Transaction.Transactions` iterator type and compiler lowering.

A StoreKit-specific iterator bridge may be substantially simpler than general protocol/existential async-sequence machinery.

If that works, extract reusable ABI primitives only after they are demonstrated by the concrete implementation.

## UI interop is not the hard part

StoreKit's UIKit purchase overload accepts a `UIViewController`.

That object can already be obtained through objc2. Passing an Objective-C object reference through a Swift ABI call should be much simpler than reproducing SwiftUI purchase views.

Therefore the framework should prefer the UIKit-confirmation StoreKit purchase API rather than making StoreKit support depend on SwiftUI.

## App Store compliance

Swift ABI stability does **not** automatically make every Swift symbol a supported App Store API.

The implementation must call only ABI-public entrypoints corresponding to documented public StoreKit API.

Before shipping:

- inspect the public StoreKit module interface shipped in the supported Xcode SDK;
- map each mangled symbol to a public declaration;
- do not call private helpers, underscored declarations, or accidental implementation symbols;
- verify linking does not depend on private frameworks;
- archive/validate a representative app.

Direct ABI access is acceptable only as another calling mechanism for the same public API, never as a way to reach private StoreKit internals.

## Required feasibility experiments before implementation planning

### Experiment 1 — SDK surface inspection

On a current Xcode installation:

- locate StoreKit public `.swiftinterface` files;
- record the exact declarations and availability for:
  - `Product`;
  - `Product.products(for:)`;
  - UIKit `purchase(confirmIn:options:)`;
  - `PurchaseResult`;
  - `VerificationResult`;
  - `Transaction`;
  - `Transaction.updates`;
  - `Transaction.finish()`;
- inspect public exported symbols and stable manglings.

### Experiment 2 — Swift compiler lowering oracle

Compile minimal temporary Swift source **outside the framework source tree** solely as a research oracle, or inspect equivalent compiler-generated IR from an isolated experiment.

The repository guarantee is no Swift source in the framework/product. Research tooling may inspect compiler output without making Swift a framework build dependency.

Capture:

- LLVM IR call signatures;
- SIL where useful;
- final arm64 assembly;
- metadata/witness arguments;
- async entry/resume behavior.

If the zero-Swift research policy is later intended to forbid even temporary external research fixtures, use SDK interfaces and compiler tests/upstream ABI documentation instead.

### Experiment 3 — synchronous Swift-call proof

Before StoreKit, construct the smallest public Swift ABI call with known value types using a `swiftcall` thunk and verify:

- arm64 device;
- arm64 simulator;
- ownership/destruction;
- no Swift source in the resulting framework repository.

### Experiment 4 — Swift value metadata

Prove from Rust/C ABI that the implementation can:

- resolve known Swift type metadata;
- inspect value witness size/alignment/stride;
- allocate correctly aligned opaque storage;
- initialize/copy/destroy a nontrivial value correctly.

### Experiment 5 — async proof

Call a small public async Swift API through the same mechanism and resume into Rust without a custom Swift source bridge.

Measure:

- thunk instructions;
- allocations;
- task/context allocations intrinsic to Swift;
- incremental allocations added by the Rust adapter.

### Experiment 6 — StoreKit product retrieval

Use StoreKit Test or sandbox to request products.

Validate:

- Swift string/collection construction;
- generic metadata/witness arguments;
- async error/result handling;
- Array<Product> iteration;
- Product field access;
- destruction with no leaks.

### Experiment 7 — purchase lifecycle

Using a StoreKit test configuration:

- purchase via UIKit confirmation overload;
- distinguish user-cancelled / pending / success;
- unwrap `VerificationResult<Transaction>`;
- read transaction fields;
- call `finish()`;
- prove exactly-once ownership and completion behavior.

### Experiment 8 — transaction updates

Bridge `Transaction.updates` into a Rust stream/callback abstraction.

Test cancellation, teardown, app background/foreground behavior, repeated events, and no leaked Swift task/iterator state.

## Recommended initial implementation architecture

Do not expose Swift ABI details to application code.

Possible internal layering:

```text
ios-rust-storekit public Rust API
        |
        v
Rust StoreKit ownership/state adapter
        |
        v
swift_abi internal primitives
        |
        +-- metadata/value-witness helpers via standard ABI
        |
        +-- generated/fixed swiftcall thunks
        |
        v
public StoreKit Swift ABI
```

The `swift_abi` layer should be internal/unstable initially.

Every primitive added to it must have a concrete StoreKit consumer and an ABI test.

## Preliminary conclusion

Direct Rust-to-StoreKit-2 interoperability appears technically feasible without a Swift source layer and without a heavyweight bridge runtime because:

1. Swift's ABI is stable on Apple platforms.
2. The Swift runtime/standard library already ship with the OS.
3. Swift's calling convention, metadata, value witnesses, generics and mangling are substantially documented.
4. Clang already exposes `swiftcall` support that can serve as an ABI adapter and correctness oracle.
5. The required overhead can remain native-call-scale rather than serialization/runtime-bridge-scale.

However, **pure stable Rust is not currently sufficient by itself to emit arbitrary Swift calls**, because Rust's native Swift ABI support is still experimental/unimplemented.

The best first feasibility path is therefore:

1. use Rust for all framework state/API logic;
2. use C-ABI calls into documented Swift runtime helpers where appropriate;
3. use microscopic Clang `swiftcall` thunks for the minimum StoreKit entrypoints;
4. inspect generated arm64 code;
5. migrate individual thunks to Rust/assembly/LLVM only when doing so improves maintainability or measured cost;
6. eventually adopt stable Rust `abi_swift` if/when it exists.

This preserves the project's core goals: zero Swift source, no bridge runtime, App Store public APIs only, and near-native overhead.
