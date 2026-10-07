# Swift C++ Wrapper Cost Model

Research date: 2026-10-07

## Executive conclusion

Swift's generated C++ interoperability headers are extremely valuable as:
- a compiler-authored ABI oracle;
- a source of correct symbol names;
- a source of correct value ownership/copy/destroy behavior;
- a convenient representation for many fixed-layout Swift values and classes.

However, they are **not automatically an acceptable zero-overhead shipping representation** for every Swift type.

The most important performance finding is:

> Swift's official C++ interoperability layer heap-boxes resilient Swift structs.

The official guide states that, for resilient structures, operations including:
- returning a resilient value from a Swift call;
- returning a resilient value from a property getter;
- creating a resilient value from C++;
- copying the C++ proxy;

allocate new heap storage.

That is potentially extra overhead relative to a native Swift caller, which can often work with opaque caller-provided storage/value witnesses without adding an additional C++ proxy box.

Therefore the project should distinguish:

1. **generated C++ as correctness oracle**;
2. **generated C++ as final implementation backend**.

Use (2) only when measurements show it does not violate the framework's incremental-overhead target.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/CppInteroperability/UserGuide-CallingSwiftFromC++.md

---

# 1. Fixed-layout Swift structs

For a Swift structure whose layout is fixed, the generated C++ representation is an opaque C++ class whose size/alignment match the Swift structure.

The C++ proxy can therefore hold the Swift value inline rather than introducing a separate bridge object.

The generated interface supplies:
- construction;
- copy/move/destruction behavior;
- property accessors;
- methods.

## Framework implication

For fixed-layout Apple Swift values:

> generated C++ may be an acceptable final backend if optimized code shows no extra allocation/copy versus an equivalent Swift caller.

Still measure:
- copy count;
- destructor/value-witness calls;
- inlining;
- register/indirect-result behavior.

Do not assume every generated C++ convenience method is free.

---

# 2. Resilient Swift structs

Swift resilience means clients cannot bake the type's size/layout into their ABI.

The official C++ interop implementation solves this by **boxing the resilient value on the heap**.

The guide explicitly says the following allocate:
- returning a resilient struct from a Swift function/method;
- returning one from a getter;
- constructing one through the generated static initializer;
- copying the C++ proxy.

A fixed-layout struct that contains a resilient struct may also become boxed in the generated C++ representation.

## Why this matters

This project has a stronger performance target than "C++ can call Swift correctly."

For a frequently returned Apple value such as a StoreKit/Translation/FoundationModels domain struct, automatic proxy boxing could mean:

```text
native Swift caller
    -> caller-provided/ABI-managed value storage

Rust via generated C++ proxy
    -> heap allocate C++ resilient-value box
    -> Swift value inside box
```

If that extra allocation exists only because of the C++ interoperability representation, it is exactly the kind of incremental language-bridge overhead the framework is intended to avoid.

## Decision

Generated C++ resilient proxies are:
- **approved as research oracles**;
- **not automatically approved as shipping value representation**.

Before using one in the final framework:
1. benchmark allocation behavior;
2. compare to an equivalent Swift caller;
3. inspect optimized assembly;
4. determine whether a direct opaque-storage ABI path removes the allocation.

---

# 3. Preferred direct representation for hot resilient values

The Swift ABI already provides the concepts needed to manipulate resilient values without knowing their layout statically:

- type metadata;
- value witness table;
- runtime size;
- alignment;
- stride;
- initialize/copy;
- initialize/take;
- destroy.

This matches the existing `MINIMUM_SWIFT_ABI_PRIMITIVES.md` direction.

A likely Rust internal representation is conceptually:

```text
SwiftTypeMetadata
  -> value witnesses
  -> runtime size/alignment

OpaqueSwiftValue
  -> correctly aligned caller-owned storage
  -> metadata reference
  -> explicit initialization/destruction
```

The storage strategy can then be selected independently:
- stack for compile-time-bounded/small known cases where legal;
- caller arena/scratch region;
- inline small buffer plus fallback;
- Rust allocator;
- Swift/runtime allocation if required by the type/operation.

The key is that allocation is a framework decision based on the ABI requirement, not forced by a generic C++ proxy class.

## Research requirement

Do not assume stack allocation is always possible merely because runtime size is known.

The implementation must handle:
- runtime alignment;
- lifetime across async suspension;
- move/copy rules;
- pointer stability;
- ownership;
- escaping values.

---

# 4. C++ wrappers remain valuable for value-witness correctness

Even when not shipped directly, generated C++ wrappers provide a compiler-authored reference for:

- when a value is copied;
- when it is moved;
- when it is destroyed;
- how metadata is resolved;
- how methods/accessors receive the value;
- how resilient results are initialized;
- how associated values are extracted;
- how generic type metadata/witnesses are forwarded.

## Recommended workflow

For a target type such as StoreKit `Product`:

1. generate or otherwise obtain the C++ compatibility representation from the SDK module interface;
2. compile a tiny use-site;
3. inspect LLVM IR/ARM64 assembly;
4. identify allocation/value-witness calls;
5. implement a direct Rust/Clang ABI version;
6. prove that the direct version has equivalent ownership but fewer/no proxy allocations;
7. retain a regression test against the compiler-authored reference.

This is safer than deriving the direct path only from prose ABI documentation.

---

# 5. Swift String interoperability

The generated C++ interop layer supports `swift::String`.

The official guide documents:
- implicit construction from a C string/string literal;
- explicit construction from `std::string`;
- conversion from Swift String to `std::string`;
- Objective-C++ conversion to `NSString *`.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/CppInteroperability/UserGuide-CallingSwiftFromC++.md

## Performance caution

Convenient conversion does not imply zero-copy.

In particular:
- Rust UTF-8 -> `std::string` -> Swift String can introduce an avoidable intermediate allocation/copy;
- Swift String -> `std::string` -> Rust String can do the same;
- Swift String -> NSString may involve bridging behavior that needs measurement.

## Preferred framework strategy

For hot paths, investigate the compiler/runtime entrypoints underlying:
- UTF-8 construction;
- borrowed UTF-8 access;
- native NSString bridging.

Provide separate APIs where semantics differ:
- borrowed native Swift string;
- owned Swift string;
- copy-to-Rust;
- bridge-to-NSString.

Do not hide copying behind a generic `String` conversion API.

## Static strings

Where an Apple API accepts a stable/static identifier, investigate whether:
- native NSString constants;
- static Swift string representation;
- cached Swift values;

can avoid repeated construction.

---

# 6. Swift Array interoperability

The generated C++ layer supports:

```cpp
swift::Array<T>
```

including:
- initializer-list construction;
- count;
- indexed access;
- iteration;
- mutation.

The official guide also documents conversions to/from `std::vector`, and explicitly notes that these copy elements.

## Framework rule

Do **not** use `std::vector` as an intermediate between Rust `Vec<T>` and Swift Array unless a benchmark/implementation constraint justifies it.

Bad path:

```text
Rust Vec
 -> C++ std::vector copy
 -> Swift Array copy
```

Preferred paths to research:

```text
Rust values
 -> direct Swift Array construction
```

or, where API permits:

```text
Rust buffer
 -> borrowed/native collection view
```

or avoid constructing Swift collections entirely by selecting a native Apple overload.

## StoreKit implication

`Product.products(for:)` takes a generic Collection of String.

Even if the generated C++ `swift::Array<swift::String>` implementation is convenient, benchmark:
- Rust identifiers -> Swift String construction;
- Array storage allocation;
- per-element retains/copies;
- destruction.

This operation is not likely a hot microsecond-scale loop in real IAP use, so correctness/maintainability may outweigh eliminating a single allocation, but the cost should remain explicit.

---

# 7. Swift Optional

The generated C++ layer supports:

```cpp
swift::Optional<T>
```

with:
- construction;
- boolean presence test;
- value extraction;
- mutation helpers.

For fixed/simple payloads, this may be an acceptable direct representation.

For resilient payloads, the same boxing/copy concerns may propagate.

## Framework strategy

For internal ABI:
- prefer metadata/value-witness-aware optional handling;
- use generated C++ behavior as oracle;
- do not raw-switch on memory discriminator unless ABI guarantees it for the concrete type.

---

# 8. Swift classes are a different case

Swift class references are naturally reference values.

The generated C++ representation provides ARC semantics through:
- copy constructor;
- assignment;
- destructor.

Unlike resilient struct boxing, retaining a class reference is not inherently an artificial bridge allocation; the underlying object is already a heap/reference-semantic Swift object.

## Framework implication

Generated C++ representation may be much more acceptable for:
- `TranslationSession`;
- `LanguageModelSession`;
- `WidgetCenter`;
- `FinanceStore`;
- other Apple Swift classes.

Still audit:
- redundant retains/releases;
- C++ wrapper copies;
- whether Rust can store the raw Swift object reference with explicit native retain/release more cheaply.

The final Rust API should not expose C++ wrapper classes.

---

# 9. Generic representation limits

Swift C++ interoperability has partial generic support.

Current documented limitations include:
- generic structs with constraints are not generally representable;
- generic enums with constraints are not generally representable;
- generic classes are not supported;
- generic functions/methods have limitations;
- protocol values and closures have major gaps.

This is important because high-priority Apple APIs use exactly these difficult shapes:
- StoreKit generic constrained product lookup;
- `VerificationResult<T>`;
- AsyncSequence;
- protocol-based App Intents.

Therefore generated C++ can reduce ABI work, but it cannot be the only backend.

---

# 10. Generated C++ storage should not leak into Rust API design

Do not design public Rust types around the current C++ proxy representation.

For example, do not expose:

```text
struct Product {
    cxx_box: ...
}
```

as the permanent architecture.

Instead expose Rust semantic/native handles whose backend can evolve:

```text
Product
   -> internal SwiftValue storage/backend
```

Possible backend changes over time:
- generated C++ proxy;
- direct opaque Swift value;
- direct rustc Swift ABI;
- optimized fixed-layout specialization.

Application code should not care.

---

# 11. Cost classification for generated C++ interop

## Usually promising

- top-level/direct synchronous function wrappers;
- fixed-layout structs;
- enums with supported representation;
- Swift class references;
- primitive arguments/results;
- Objective-C object arguments/results;
- synchronous property accessors;
- String/Array helpers as correctness prototypes.

## Must benchmark carefully

- resilient structs;
- structs containing resilient fields;
- repeated String conversions;
- Array construction/conversion;
- proxy copies;
- nested Optionals/generic values.

## Not a complete solution today

- async/throws in general;
- AsyncSequence;
- constrained generic APIs;
- protocol/existential values;
- closures;
- App Intents-style Rust-defined protocol types;
- compiler metadata.

---

# 12. Updated backend policy

The earlier `CLANG_SWIFT_ABI_BACKENDS.md` conclusion is refined:

### Generated C++ is preferred when BOTH are true

1. it can represent the public API;
2. its generated representation does not add unacceptable allocation/copy overhead for that use case.

### Generated C++ is an oracle-only path when

- resilient-value boxing creates avoidable overhead;
- API is async/constrained-generic/protocol-based;
- generated wrapper performs copies not required by the native Swift ABI.

### Direct ABI is preferred when

- native Swift could operate on caller-provided opaque storage;
- proxy boxing/copying is measurable;
- the call is common/hot enough to matter;
- direct metadata/value-witness handling remains maintainable.

---

# 13. Concrete benchmark requirements

For every high-priority Swift domain type, record:

| Metric | Swift caller | Generated C++ | Direct Rust ABI |
|---|---:|---:|---:|
| heap allocations constructing value | | | |
| allocations returning value | | | |
| copies | | | |
| moves/takes | | | |
| destroy calls | | | |
| retains/releases | | | |
| bytes copied for strings | | | |
| bytes copied for arrays | | | |
| instruction count at call boundary | | | |

First target types:
- StoreKit Product;
- StoreKit Transaction;
- TranslationSession.Response;
- FoundationModels response;
- Swift String;
- Array<String>;
- Optional<String>.

Do not optimize based on the C++ representation alone. Compare against a native Swift caller.

---

# Final conclusion

Compiler-generated C++ interoperability is a major **correctness accelerator**, but not a universal zero-overhead backend.

The most important exception is resilient Swift value types, where official C++ interop deliberately trades performance for representability by heap-boxing values.

For this framework's stronger goal, the likely optimal combination is:

```text
compiler-generated C++ wrappers
    -> ABI oracle + low-frequency/simple backend

direct metadata/value-witness storage
    -> high-performance resilient values

Clang swiftcall/swiftasynccall
    -> actual Swift call lowering

Rust
    -> application-facing ownership/state/API
```

This hybrid approach is still far smaller and safer than implementing a general Swift runtime, while preserving the project's goal of eliminating avoidable language-bridge allocations.
