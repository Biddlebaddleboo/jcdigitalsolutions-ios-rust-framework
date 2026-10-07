# Swift Runtime Primitives for a Rust Caller

Research date: 2026-10-07

## Executive conclusion

The project should not build its own Swift object runtime.

Swift's stable ABI includes:
- reference counting;
- allocation;
- type metadata;
- value witness tables;
- protocol conformance machinery;
- generic metadata support;
- array/value helper operations;
- error objects.

A significant subset of the compiler/runtime entry points used for those operations use the ordinary C calling convention and are available on ABI-stable Apple platforms.

This is favorable for Rust:
- ordinary C-ABI runtime calls can be declared directly;
- Swift calling-convention thunks are only required for actual Swift functions/methods and the runtime functions explicitly using SwiftCC;
- Rust can reuse Swift's existing retain/release and value-witness machinery instead of reproducing it.

However:

> Do not bind every exported `swift_*` symbol merely because it exists.

The implementation must distinguish:
1. stable/documented ABI structure or runtime entry point;
2. compiler support entry point explicitly marked available for deployed Apple targets;
3. internal/private runtime implementation detail.

Only categories 1–2 are candidates for the framework.

Primary references:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst
https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/RuntimeFunctions.def

---

# 1. Swift runtime API is part of the ABI

Swift's ABI stability documentation explicitly identifies the Swift runtime API as part of the language ABI because compiler-generated programs call into the runtime for:
- reference counting;
- dynamic casting;
- reflection;
- generic metadata;
- protocol conformance;
- memory management.

This is not analogous to reverse-engineering a private Apple service.

It is part of what independently compiled Swift code relies on for binary compatibility.

That said, individual runtime symbols still need a stability/availability audit.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

---

# 2. Native Swift class/object ownership

The current Swift runtime function database includes:

```c
void *swift_retain(void *);
void swift_release(void *);
```

with:
- ordinary C calling convention;
- `AlwaysAvailable` runtime availability;
- explicit refcounting semantics.

It also includes variants for:
- multiple retains/releases;
- unknown objects;
- bridge objects;
- weak/unowned reference storage;
- Swift Error objects.

Reference:
https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/RuntimeFunctions.def

## Framework direction

A Rust wrapper around a native Swift class should conceptually behave like:

```text
struct SwiftRetained<T> {
    ptr: NonNull<T>,
}
```

with:
- clone -> native `swift_retain`;
- drop -> native `swift_release`.

Do **not** put Swift class references inside `Arc` merely to obtain shared ownership.

The Swift runtime already supplies the reference counting.

## Objective-C overlap

On Apple platforms, Swift class metadata is fundamentally interoperable with the Objective-C runtime at the class-metadata level.

But do not assume:
- `objc_retain` and `swift_retain` are universally interchangeable for every Swift class/reference representation;
- every Swift class method has an Objective-C selector.

Use the ownership operation dictated by the actual reference kind.

---

# 3. Native Swift object allocation

The runtime function database includes C-ABI object allocation/deallocation operations such as:

```text
swift_allocObject
swift_deallocObject
swift_deallocUninitializedObject
```

and helpers for stack/static object initialization.

These are compiler support primitives.

## Framework rule

Do not allocate Apple framework classes manually through `swift_allocObject` merely because the runtime exposes it.

For Apple-defined types, call the type's public initializer/factory API.

Direct object allocation becomes relevant primarily for future **Layer 2** work where Rust needs to define a Swift-compatible class/type and must implement compiler-like construction semantics.

Layer 1 should consume Apple-created objects.

---

# 4. Type metadata is the central runtime type handle

Swift type metadata contains the runtime information needed for:
- kind;
- layout;
- fields/cases;
- generic instantiation;
- value witnesses;
- protocol conformances;
- reflection/runtime operations.

The stable ABI documentation defines key metadata layout relationships.

Most importantly:

> the value witness table pointer is stored one pointer-sized word immediately before the metadata pointer.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABI/TypeMetadata.rst

## Type kind

The ABI documentation defines metadata kinds for:
- class;
- struct;
- enum;
- optional;
- tuple;
- function;
- protocol/existential;
- metatype;
- Objective-C wrapper;
- others.

This allows an internal Rust metadata wrapper to validate expected categories in debug/validation paths.

Do not insert runtime kind checks into every hot operation once a typed handle has been validated.

---

# 5. Value witness tables are the correct abstraction for opaque values

Every concrete Swift type has value witnesses describing operations such as:
- size;
- stride;
- alignment;
- initialize-with-copy;
- initialize-with-take;
- assign-with-copy;
- assign-with-take;
- destroy;
- extra-inhabitant/enum operations where relevant.

This is precisely how a client can manipulate a resilient Swift value without knowing its fields/layout at compile time.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/ABIStabilityManifesto.md

## Framework direction

The core internal abstraction should be conceptually:

```text
SwiftMetadata<T>
    |
    +--> ValueWitnesses
           |
           +--> size
           +--> alignment
           +--> stride
           +--> copy
           +--> move/take
           +--> destroy
```

Use those operations for resilient values rather than:
- C++ proxy-box copies;
- `memcpy`;
- guessed layouts;
- generic serialization.

## Safety rule

Never use `memcpy` on an opaque Swift value merely because its current representation appears trivial.

Only use bitwise copy/move when the value witness flags guarantee the operation.

---

# 6. Caller-owned opaque value storage

A direct Rust ABI implementation can use metadata/value witnesses to size and align storage for a Swift value.

Potential internal representation:

```text
OpaqueSwiftValue {
    storage,
    metadata
}
```

with initialization state tracked explicitly.

## Important states

A storage region can be:
- uninitialized;
- initialized/owned;
- moved-from;
- destroyed.

Rust must not call:
- destroy twice;
- copy from uninitialized storage;
- take from a moved/destroyed value.

This is a natural typestate/unsafe-core problem.

## Allocation strategy

Do not hard-code "all opaque values heap allocate."

Possible strategies:
- fixed inline storage for known fixed-layout types;
- stack/scratch allocation in synchronous generated thunk;
- bump/arena allocation for short-lived batches;
- Rust heap for values escaping their call frame;
- Swift-owned box only when semantics require it.

Async suspension is a key boundary: result/value storage that survives suspension needs a stable lifetime.

---

# 7. Runtime C calling convention is a major simplification

The Swift runtime function database records the calling convention for each compiler support entry point.

Many important primitives use ordinary C calling convention, including:
- `swift_retain`;
- `swift_release`;
- `swift_allocObject`;
- `swift_deallocObject`;
- `swift_projectBox`;
- several array copy/take/destroy operations;
- function-type metadata constructors.

Reference:
https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/RuntimeFunctions.def

## Consequence

These can be called directly from stable Rust `extern "C"` declarations, subject to:
- exact signature;
- OS availability;
- stable ABI status;
- linkage.

No `swiftcall` thunk is needed merely to retain a Swift object or call one of these C-ABI runtime helpers.

---

# 8. Not every runtime helper uses C ABI

Examples in the runtime database include some entries marked `SwiftCC`, such as certain:
- box allocation paths;
- error/reporting paths;
- metadata operations.

Do not assume every `swift_*` function is C-callable.

The code generator must record for every runtime symbol:
- symbol name;
- calling convention;
- availability;
- exact signature;
- stability classification.

If a needed runtime helper uses SwiftCC:
- either call through a tiny Clang/LLVM thunk;
- or use an alternative C-ABI/value-witness path if available.

---

# 9. Arrays have low-level runtime value operations

The current runtime function database contains ordinary C-ABI array operations for contiguous arrays of arbitrary Swift element type, including operations conceptually equivalent to:
- initialize with copy;
- initialize with take;
- assign with copy/take;
- destroy a sequence.

These accept:
- destination/source storage;
- element count;
- element type metadata.

Reference:
https://github.com/swiftlang/swift/blob/main/include/swift/Runtime/RuntimeFunctions.def

## Important distinction

These helpers manipulate **arrays of values in memory**.

They are not automatically the same thing as constructing the high-level Swift `Array<T>` standard-library collection object.

Do not conflate:
- an array of T values in opaque storage;
- Swift.Array<T>'s native storage/container representation.

For high-level `Array<T>` construction, generated C++/Swift stdlib ABI or compiler-emitted code may still be preferable as an oracle/backend.

---

# 10. Swift String should remain compiler/stdlib-backed initially

Although Swift's standard library ABI is stable, Swift String has a sophisticated optimized representation.

Do not manually implement String storage.

Preferred order:

1. use compiler-generated C++ `swift::String` construction as correctness oracle;
2. inspect which stable standard-library entrypoints it calls;
3. if the C++ wrapper adds avoidable copies, bind the underlying stable entrypoints or generate a direct thunk;
4. benchmark UTF-8 construction/bridging.

Avoid relying on underscored/internal String symbols merely because they are visible in a current OS.

## Objective-C bridge

Where the target Apple API accepts Objective-C `NSString`, skip Swift String entirely.

Where a genuine Swift API requires Swift String, construct Swift String once at the ABI boundary.

---

# 11. Swift Error ownership

The runtime includes C-ABI retain/release operations for Swift Error objects.

Throwing synchronous calls return errors through the Swift error-result ABI register, modeled by Clang `swift_error_result`.

The Rust side needs an owned error handle whose Drop uses the correct Swift error release operation.

## Error conversion policy

Do not immediately stringify every error.

Preserve:
- native Swift Error object;
- type metadata;
- bridged NSError when legitimately available;
- optional lazy display/debug description.

This avoids:
- string allocation;
- information loss;
- hidden conversion cost.

---

# 12. Protocol conformance and generic metadata

Swift runtime/metadata machinery supports:
- protocol conformance lookup/registration;
- generic metadata instantiation;
- witness-table resolution.

These capabilities are part of compiler/runtime ABI design.

However, these are more delicate than retain/release.

## Framework direction

For **Layer 1**:
- prefer compiler-emitted/static metadata accessors and witness references from the target module;
- avoid global dynamic conformance lookup on every call;
- resolve/cache immutable metadata/witness handles once when safe.

For **Layer 2**:
- Rust-defined Swift protocol conformances will eventually require emitting conformance descriptors/witness tables, not merely calling a dynamic lookup helper.

## Performance

Metadata/witness lookup should be:
- setup-time/lazy-once where possible;
- not repeated in hot loops.

Generic calls should pass already-resolved metadata/witness pointers.

---

# 13. Stable ABI versus implementation symbols

The research must maintain three labels.

## A — ABI contract

Examples:
- metadata layout;
- value-witness table relationship;
- calling convention;
- mangling rules;
- Swift runtime reference-count semantics.

Safe foundation for long-term design.

## B — compiler runtime support entry point

Examples:
- `swift_retain`;
- `swift_release`;
- compiler support functions explicitly recorded with stable/target availability.

Potentially callable directly, but each symbol should be verified against:
- current runtime headers/database;
- minimum deployment target;
- ABI availability.

## C — runtime implementation detail

Examples may include:
- private C++ symbols;
- underscored helper functions not intended as ABI;
- internal reflection/debug helpers;
- implementation-specific standard-library symbols.

Do not bind these in shipping code.

The mere presence of a symbol in:
- dyld export list;
- SDK binary;
- Swift source repository;

is not enough to classify it as A/B.

---

# 14. Deployment availability must be modeled

The runtime database distinguishes availability categories such as:
- always available;
- concurrency-era availability;
- opaque-type-era availability;
- newer runtime feature availability.

A runtime wrapper must not assume the newest iOS runtime entrypoints exist on the framework's entire deployment range.

Each direct runtime binding needs:
- minimum OS version;
- weak/dynamic lookup only when supported and necessary;
- fallback/compiler-generated implementation if required.

Do not use availability checks in hot paths once a process-level backend capability has been resolved.

---

# 15. Runtime symbols should be statically declared where stable

For stable ABI runtime functions such as native retain/release, prefer normal linker declarations over repeated `dlsym`.

Reasons:
- lower overhead;
- compile/link validation;
- no string lookup;
- clearer dependency.

Use dynamic symbol lookup only where:
- optional version-dependent runtime support requires it;
- public ABI permits that usage;
- weak linking is insufficient.

Never use dynamic lookup to reach private symbols.

---

# 16. Proposed Rust unsafe-core modules

This is not an implementation plan yet, but research supports the following eventual separation:

```text
swift_abi/
  runtime/
    refcount
    error
    availability
  metadata/
    metadata
    value_witness
    generic
    conformance
  value/
    storage
    state
    copy_take_destroy
  call/
    sync
    async
    thunk_backend
```

Each should expose a narrow safe/typed layer over a documented unsafe core.

Application-facing framework code should never manipulate raw Swift metadata pointers directly.

---

# 17. First runtime primitives to validate experimentally

On macOS/Xcode, validate in this order.

## R1 — class retain/release

Using a public Swift class:
- obtain class object from compiler-generated call;
- retain with `swift_retain`;
- release;
- verify lifetime/deinit under sanitizer/instrumentation.

## R2 — metadata/value witnesses

For a known resilient public Swift struct:
- obtain metadata;
- read value-witness pointer at documented location;
- read size/alignment/stride;
- allocate correctly;
- copy;
- take;
- destroy.

Compare against generated C++ wrapper.

## R3 — direct resilient result storage

Call a synchronous Swift function that returns a resilient value indirectly.

Place result directly into Rust-owned opaque storage.

Verify:
- no C++ proxy box;
- no extra allocation relative to native Swift;
- correct destruction.

## R4 — error object

Call a synchronous throwing Swift function:
- capture Swift error register through Clang thunk;
- retain/transfer ownership correctly;
- bridge to NSError only when supported;
- release with Swift error runtime operation.

## R5 — String

Construct Swift String from Rust UTF-8 using:
1. generated C++ path;
2. lower-level direct path derived from compiler output.

Measure copies/allocations.

## R6 — Array<String>

Construct and destroy array used for a real Apple API call.

Measure:
- allocations;
- element copies;
- String copies;
- witness calls.

---

# Final conclusion

The low-level runtime research strongly supports the project's central approach:

> Use Apple's existing Swift runtime directly; do not implement a Swift runtime in Rust.

Rust's responsibilities should be limited to:
- correctly representing owned/borrowed handles;
- caller-owned opaque value storage;
- invoking stable runtime operations;
- supplying metadata/witness pointers;
- calling public Swift APIs through compiler-correct ABI thunks.

This can remain much closer to native Swift execution than any serialization bridge or language-runtime proxy.

The hardest remaining problems are not reference counting or opaque value management. They are:
- high-level ABI lowering for complex generics;
- async/resume ABI;
- Rust-defined Swift protocol/type metadata;
- compiler-generated framework-specific metadata such as App Intents.

Those should continue to be attacked only as concrete high-priority Apple APIs require them.
