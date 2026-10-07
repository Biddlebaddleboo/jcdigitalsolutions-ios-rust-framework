# Clang / LLVM Backends for Direct Swift ABI Calls

Research date: 2026-10-07

## Executive conclusion

The Swift interoperability backend does **not** need to start from handwritten ARM64 assembly.

Clang and LLVM already contain substantial first-class machinery for the Swift calling convention:

- `swiftcall`;
- `swift_context`;
- `swift_error_result`;
- `swift_indirect_result`;
- `swiftasynccall`;
- `swift_async_context`;
- LLVM `swiftcc` / `swifttailcc`;
- target-specific Swift registers on Apple ARM64.

Even more importantly, Swift's official C++ interoperability system can generate compiler-authored C++ wrappers for public Swift APIs, including resilient Swift structs/enums/classes, Strings, Arrays and Optionals. Swift's design explicitly says these bindings can be generated **retroactively from a Swift module interface**, without needing the library's original source.

That creates a better hierarchy for the first ABI implementation:

1. **Use generated Swift-to-C++ bindings directly when the Apple SDK module/API is representable.**
2. **Use those generated bindings as the ABI oracle even when the final public Rust layer does not expose C++.**
3. **Use small Clang `swiftcall`/ `swiftasynccall` thunks for API shapes not represented by generated C++ bindings.**
4. **Generate LLVM IR/object thunks where Clang's source-level attributes are insufficient or where generation is simpler than C++.**
5. **Use hand-written ARM64 assembly only for proven gaps/hot paths.**
6. **Migrate to direct rustc Swift ABI support if/when it becomes mature.**

This preserves:
- zero Swift source in the repository;
- no bridge runtime;
- no serialization layer;
- native-call-scale overhead;
- compiler-authored ABI lowering wherever possible.

---

# 1. What Clang `swiftcall` actually solves

Clang documents `swiftcall` as the Swift function calling convention.

Important limitation:

> `swiftcall` performs the **second phase** of Swift ABI lowering.

The C/C++ declaration still has to accurately represent the output of Swift's **high-level lowering**, including:
- direct versus indirect arguments/results;
- generic metadata arguments;
- protocol witness arguments;
- context/self;
- error result.

Reference:
https://clang.llvm.org/docs/AttributeReference.html

## Consequence

This is correct:

```text
Swift source/API signature
   ↓
high-level Swift ABI lowering
   ↓
C/C++ declaration describing lowered signature
   ↓
Clang swiftcall
   ↓
target register/stack assignment
```

This is **not** correct:

```text
copy Swift source signature into a C prototype
   ↓
add __attribute__((swiftcall))
   ↓
expect Clang to infer Swift generics/value lowering
```

Therefore manual `swiftcall` thunks still need either:
- compiler-generated reference signatures;
- Swift ABI lowering knowledge;
- generated metadata based on SDK interfaces.

This strengthens the case for using Swift's C++ header generator as the high-level lowering oracle.

---

# 2. Context, error and indirect-result support

Clang provides explicit parameter attributes for key Swift ABI treatments.

## `swift_context`

Marks a pointer/reference parameter as the Swift context/self value.

On supported targets this is passed using the special Swift context register.

For Apple ARM64 this corresponds to x20.

## `swift_error_result`

Marks the final parameter of a synchronous `swiftcall` function as the Swift error-result location.

Clang models the special error register as if C code passed a pointer-to-pointer:
- caller loads the variable into the special register;
- callee receives an address-like model;
- returned register value is written back.

On Apple ARM64 the Swift error register is x21.

A `swift_error_result` parameter must follow `swift_context`.

## `swift_indirect_result`

Marks an explicit indirect result.

It is valid for both:
- `swiftcall`;
- `swiftasynccall`.

It must appear before ordinary parameters (or after another indirect-result parameter).

On ARM64, the target ABI may use the dedicated indirect-result location, including x8 where appropriate.

Reference:
https://clang.llvm.org/docs/AttributeReference.html

## Architectural implication

Synchronous:
- instance methods;
- throwing functions;
- opaque/resilient indirect returns;

can be represented directly in C/C++ once the high-level lowered signature is known.

This should eliminate the need for assembly in the first synchronous proof.

---

# 3. Clang has low-level Swift async calling-convention support

Clang documents:

```text
swiftasynccall
swift_async_context
```

A `swiftasynccall` function:
- follows Swift's low-level async calling convention;
- returns `void`;
- may use:
  - `swift_async_context`;
  - `swift_context`;
  - `swift_indirect_result`;
- does **not** use `swift_error_result` in the same manner as synchronous `swiftcall`;
- receives guaranteed tail-call treatment for the documented return-call pattern.

Reference:
https://clang.llvm.org/docs/AttributeReference.html

The documentation explicitly warns that the calling convention is ABI-stable only on targets where Swift ABI stability has been declared.

Apple iOS/arm64 is an ABI-stable Swift platform.

## Critical limitation

This does **not** mean Clang can automatically turn:

```swift
func f() async throws -> Product
```

into a usable C declaration.

The caller still needs to know:
- the lowered async entrypoint signature;
- async context structure/lifetime;
- resume/continuation function shape;
- direct/indirect result placement;
- error delivery;
- generic metadata/witness arguments;
- actor/executor requirements.

Therefore:

> `swiftasynccall` solves the target calling convention once we know the low-level ABI shape. It does not replace the Swift compiler as a high-level async-lowering oracle.

---

# 4. Do not confuse `swift_async` with reverse Swift ABI calls

Clang also documents:
- `swift_async`;
- `swift_async_error`;
- `swift_async_name`.

Those attributes describe how a **C/Objective-C completion-handler API is imported into Swift as async**.

Example use case:

```text
Objective-C completion-handler method
        ↓
swift_async metadata
        ↓
Swift async overlay
```

That is the opposite direction from this framework's hard problem.

They are useful when auditing Apple APIs because they can reveal that a Swift async method is merely an imported native completion API. In those cases the framework should call the Objective-C API directly and eliminate Swift ABI entirely.

They are **not** the mechanism for calling a genuinely Swift-native async API from Rust.

Reference:
https://clang.llvm.org/docs/AttributeReference.html

---

# 5. LLVM already models Swift calling conventions directly

LLVM's IR calling-convention table contains:

```text
Swift = 16
SwiftTail = 20
```

corresponding to:
- `swiftcc`;
- tail-call-oriented Swift calling convention.

Reference:
https://llvm.org/doxygen/CallingConv_8h_source.html

LLVM's AArch64 lowering contains dedicated handling for:
- Swift self/context in x20;
- Swift error in x21;
- Swift async context in x22.

This means generated LLVM IR is a realistic backend option if source-level Clang declarations become awkward.

## Implication

A future build-time generator could produce tiny ABI object files directly:

```text
Rust build metadata
   ↓
generated LLVM IR
   ↓
Apple/LLVM toolchain
   ↓
native object
   ↓
Rust links object
```

No Swift source is required.

However, this should be a second implementation backend, not the first, because Clang-generated code and Swift-generated C++ headers provide easier validation and diagnostics.

---

# 6. Swift C++ interop is potentially the best synchronous ABI oracle

Swift officially supports generating a C++ header for Swift modules.

The header:
- contains C++ types representing Swift types;
- contains inline functions that call native Swift functions directly;
- uses Clang-specific Swift ABI attributes;
- does not introduce a bridge runtime;
- can model resilient Swift structs and enums using compiler-generated wrappers;
- implements Swift ownership/copy/destruction semantics.

References:
https://www.swift.org/documentation/cxx-interop/
https://github.com/swiftlang/swift/blob/main/docs/CppInteroperability/UserGuide-CallingSwiftFromC++.md

## Most important property

Swift's C++ interop design explicitly says:

> C++ bindings can be generated retroactively starting from a Swift module interface file; generation does not need to have been requested when the Swift library was built from source.

Reference:
https://github.com/swiftlang/swift-evolution/blob/main/visions/using-swift-from-c++.md

This is highly relevant to Apple SDK frameworks because modern Apple Swift frameworks ship module interfaces for ABI/module stability.

### Research hypothesis

For representable public StoreKit/Translation/FoundationModels APIs:

```text
Apple SDK .swiftinterface
        ↓
Swift compiler C++ compatibility-header generation
        ↓
compiler-authored ABI wrappers
        ↓
tiny C ABI facade callable from Rust
```

could avoid manually reproducing a significant portion of Swift ABI lowering.

This must be tested against the actual current iOS SDK on macOS/Xcode.

---

# 7. Zero-Swift-source invariant remains intact

Using the Swift compiler as an **interface translator/code-generation tool** does not require adding Swift source to this repository if the input is Apple's SDK module interface.

Potential build/research artifacts:
- Apple-owned `.swiftinterface`;
- generated C++ header;
- C++/C ABI shim;
- object file.

No repository-authored `.swift` file is required.

The framework rule remains:

- no Swift source;
- no generated Swift source;
- no app logic written in Swift.

Whether invoking `swiftc` solely for header generation is acceptable as a permanent build dependency is a separate architectural decision.

The first proof should test it before deciding.

---

# 8. Swift compiler exposes the required header-generation mode

Swift's compiler exposes:

```
-emit-clang-header-path <path>
```

described as emitting an Objective-C and C++ header.

With C++ interoperability enabled, the generated header exposes supported Swift APIs.

Typical source-module invocation documented by Swift is conceptually:

```
swiftc -frontend -typecheck   ...   -module-name Module   -cxx-interoperability-mode=default   -emit-clang-header-path Module-Swift.h
```

Reference:
https://www.swift.org/documentation/cxx-interop/project-build-setup/

The compiler also has a frontend mode for typechecking modules from `.swiftinterface` input.

The exact supported invocation for retroactively producing a C++ header from an Apple SDK module interface still needs to be proven on Xcode.

Do not bake an undocumented command line into the framework until that experiment succeeds.

---

# 9. C++ interop coverage is broad but incomplete

Swift's current C++ interop status documents support for:

## Functions/methods

Supported:
- top-level Swift functions;
- Swift methods;
- primitive parameters/results;
- Swift struct/enum/class parameters/results;
- Objective-C object parameters/results;
- `inout`.

Not currently represented:
- Swift closure parameters/results;
- Swift protocol-typed parameters/results;
- variadic functions;
- multiple return values.

## Structs

Supported:
- fixed-layout structs;
- resilient/opaque structs;
- copy/destroy semantics;
- initializers except throwing initializers.

## Enums

Supported:
- fixed-layout enums;
- resilient/opaque enums;
- copy/destroy;
- creation;
- raw values;
- some associated-value cases.

Not fully supported:
- indirect enums;
- all associated-value shapes.

## Classes

Supported:
- class reference values;
- ARC semantics via generated C++ copy/destructor;
- initializers except throwing initializers.

Known limitation:
- class method/virtual dispatch support is incomplete in some current cases.

## Generics

Partial:
- generic functions without generic constraints;
- generic methods without generic constraints;
- generic structs/enums without constraints and under documented parameter limits.

Not supported:
- generic classes.

## Standard library values

Supported to useful degrees:
- `String`;
- `Array<T>`;
- `Optional<T>`.

Reference:
https://github.com/swiftlang/swift/blob/main/docs/CppInteroperability/CppInteroperabilityStatus.md

---

# 10. What this means for StoreKit 2

Generated C++ interop may be very useful for:
- `Product` value representation;
- `Transaction` value representation;
- resilient field/property access;
- Swift String handling;
- Array wrappers;
- Optionals;
- copy/destroy/ownership;
- synchronous property accessors.

But it is unlikely to solve StoreKit 2 end-to-end automatically.

## Product lookup problem

`Product.products(for:)` is a **constrained generic** static function:
- generic Collection input;
- Element == String constraint;
- async/throws.

Current C++ interop generic support is incomplete for constrained generic APIs.

## Purchase

Purchase APIs are async/throws and return nested/generic Swift result types.

## Transaction streams

AsyncSequence remains outside the simple synchronous C++ wrapper story.

### Therefore

Use generated C++ binding output for every type/member it can represent, but expect:
- manual/generated async thunks;
- concrete generic metadata/witness handling;
- StoreKit-specific wrappers;

for the hardest calls.

The generated C++ header remains extremely valuable as a source of:
- type storage representation;
- metadata access;
- value-witness operations;
- symbol names;
- ownership behavior;
- synchronous accessor lowering.

---

# 11. What this means for Translation

Translation has a simpler object/value surface.

Potentially useful generated C++ coverage:
- `TranslationSession` class representation;
- synchronous properties;
- `Locale.Language`;
- `TranslationSession.Response`;
- String fields;
- destruction/ownership.

The actual:
```swift
translate(String) async throws -> Response
```
will likely still require explicit async ABI work.

This reinforces Translation as a good second proof:
- generated C++ handles value/class representation;
- custom async thunk handles one concrete async entrypoint.

---

# 12. Async appears to remain the first major custom ABI challenge

The current public C++ interop guide/status does not list ordinary Swift async functions among its supported Swift-to-C++ function shapes.

The generated-header mechanism therefore should **not** be assumed to expose StoreKit/Translation async methods directly.

This makes the likely implementation split:

```text
Synchronous Swift types/accessors
    -> compiler-generated C++ representation when possible

Async Swift entrypoint
    -> generated/hand-described low-level signature
    -> Clang swiftasynccall or LLVM swiftcc thunk

Resume/result values
    -> generated C++ type/value machinery where reusable
```

This is still substantially easier than manually recreating every Swift value layout.

---

# 13. Generated C++ wrappers should be treated as an oracle before a dependency

Even if final shipping does **not** use generated C++ headers directly, they are extremely useful during research.

For every representative Swift type/call:

1. generate the compiler's C++ header;
2. inspect the C++ wrapper;
3. compile it with Clang;
4. inspect LLVM IR and ARM64 assembly;
5. identify:
   - mangled symbol;
   - metadata accessor;
   - value-witness operations;
   - self/context treatment;
   - direct/indirect result;
   - ownership calls;
6. compare the framework's generated/manual thunk against the compiler output.

This reduces ABI guesswork.

---

# 14. Potential final backend choices

## Backend A — generated C++ wrappers + tiny C ABI facade

Best if Apple SDK module header generation works reliably and the required API is representable.

Topology:

```text
Rust
 -> extern "C" function
 -> tiny C++ inline/generated wrapper
 -> Swift ABI
 -> Apple framework
```

Incremental cost:
- one C function call unless LTO/inlining removes it;
- native Swift call;
- intrinsic Swift ownership.

With LTO or a generated C-compatible wrapper, the extra C boundary may disappear.

### Benefits
- compiler owns high-level Swift ABI lowering;
- compiler owns resilient value representation;
- least custom ABI code.

### Risks
- C++ interop feature coverage;
- Xcode/Swift compiler build dependency;
- generated header stability/source compatibility;
- async gaps.

## Backend B — Clang attributes on generated lowered signatures

Best for:
- fixed known Swift APIs;
- synchronous calls;
- concrete async call shapes after compiler-oracle analysis.

Benefits:
- microscopic generated code;
- no C++ abstraction types required in public/internal Rust interface;
- compiler handles target register assignment.

Risk:
- framework must correctly perform high-level lowering.

## Backend C — generated LLVM IR/object

Best when:
- signatures are mechanically generated;
- Clang source expression is awkward;
- async tail/resume machinery benefits from explicit IR.

Benefits:
- direct `swiftcc`;
- exact LLVM attributes.

Risks:
- LLVM/toolchain coupling;
- harder debugging.

## Backend D — ARM64 assembly

Only for:
- a missing compiler feature;
- a proven hot path;
- validating known calling sequences.

Do not use as primary backend.

## Backend E — future rustc Swift ABI

Long-term preferred if rustc obtains mature:
- Swift call convention;
- representation support;
- ABI lowering.

Internal API boundaries should allow backend replacement without application API changes.

---

# 15. Required macOS/Xcode experiments

The next practical research should run on a current Xcode installation.

## C1 — Apple module C++ header generation

Attempt to generate C++ compatibility headers from the installed SDK module interfaces for:
- StoreKit;
- Translation;
- FoundationModels.

Record:
- exact compiler command;
- whether direct module-interface input works;
- target triple and SDK requirements;
- public declarations emitted;
- diagnostics for omitted declarations.

Success here could significantly reduce implementation complexity.

## C2 — StoreKit type coverage

Inspect whether generated bindings include:
- Product;
- Transaction;
- VerificationResult;
- PurchaseResult;
- String/Array-returning properties;
- synchronous/static accessors.

Record exactly which methods are omitted.

## C3 — Translation coverage

Inspect:
- TranslationSession;
- Locale.Language;
- Response;
- synchronous properties;
- whether async methods are omitted.

## C4 — generated wrapper cost

For a simple synchronous property getter:
- compile generated header call under `-O`;
- inspect ARM64 assembly;
- compare with equivalent Swift caller;
- verify no unexpected boxing/allocation.

## C5 — manual `swiftcall`

Using a known synchronous public Swift function:
- reproduce generated call using a tiny Clang declaration;
- compare assembly;
- validate result/ownership.

## C6 — manual `swiftasynccall`

Use a minimal ABI-stable public async Swift API:
- derive lowered signature using Swift compiler IR;
- implement Clang `swiftasynccall` thunk;
- verify continuation/resume;
- return result to C/Rust callback state.

Do this before StoreKit purchase.

---

# 16. App Store and source-policy implications

Using:
- public Apple module interfaces;
- Apple's installed Swift compiler;
- generated compatibility headers;
- Clang/LLVM Swift calling-convention support;

does not by itself access a private API.

Shipping safety still requires:
- every called symbol maps to a documented public Apple declaration;
- no SPI/private interface is consumed;
- no private Swift symbols are used merely because header generation exposes/links them;
- only public module interface is used;
- Release archive validates normally.

Generated C++ headers are build artifacts, not Swift source.

The repository's zero-`.swift` invariant can remain intact.

---

# Final research conclusion

The direct Swift ABI problem is smaller than the earlier manual-thunk model suggested.

For synchronous Swift value/class operations, the compiler can potentially provide much of the ABI machinery through generated C++ interop headers.

Clang independently provides enough Swift-specific calling-convention support to implement:
- context/self;
- error result;
- indirect results;
- low-level async context/calls;

once the high-level lowered signature is known.

Therefore the framework should **not** begin by writing a custom Swift ABI implementation from scratch.

The preferred research/implementation strategy is:

```text
Apple public .swiftinterface
     |
     +--> generate compiler-authored C++ representation where possible
     |
     +--> inspect compiler lowering for unsupported calls
              |
              +--> Clang swiftcall / swiftasynccall thunk
              |
              +--> LLVM object generation if needed
              |
              '--> assembly only as last resort
```

This is more maintainable, more ABI-correct, and still compatible with the core performance objective of native-call-scale overhead with zero Swift source.
