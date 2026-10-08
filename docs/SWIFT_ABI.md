# Swift ABI Interoperability

## Repository guarantee

This repository contains no Swift source files and must not generate Swift source as part of its build.

The project may still call Apple frameworks that are internally implemented in Swift.

## Scope

Swift-ABI work exists only to reach public Apple APIs that are genuinely inaccessible through a suitable public Objective-C/C interface.

It is not a goal to implement:

- the Swift language;
- a Swift compiler;
- a general Swift standard library replacement;
- a complete Swift runtime;
- speculative support for every Swift ABI feature.

## Escalation order

Before adding Swift ABI machinery, prove:

1. there is no usable public Objective-C API;
2. there is no usable public C/CoreFoundation/Darwin API;
3. the gap is not simply a missing Rust binding;
4. reconstructing the needed convenience API from public lower-level primitives is not the better design;
5. the target Swift API is public and App-Store-compatible.

## Minimum-surface rule

Implement only the ABI features a concrete public API requires.

Potential pieces may include:

- calling conventions;
- ownership conventions;
- metadata queries;
- value-layout handling;
- closures/context;
- existentials;
- resilient structs/enums;
- async/continuation interaction.

Do not add any of these until a real API requires them.

## Compatibility

Every Swift-ABI integration must record:

- exact Apple API;
- supported OS versions;
- ABI assumptions;
- symbol/linking assumptions;
- runtime/library dependencies;
- fallback behavior;
- tests across supported deployment targets.

## Compiler evidence

- [Toolchain and SDK inventory](swift-abi/TOOLCHAIN_MANIFEST.md)
- [Swift ABI ownership and Translation lowering record](swift-abi/SYNCHRONOUS_BOUNDARY.md)
- [Synchronous scalar thunk feasibility proof](swift-abi/SYNCHRONOUS_THUNK_FEASIBILITY.md)
- [Async C++ header feasibility proof](swift-abi/ASYNC_CXX_FEASIBILITY.md)
- [Async Clang thunk feasibility proof](swift-abi/ASYNC_THUNK_FEASIBILITY.md)
- [Async task-entry feasibility audit](swift-abi/ASYNC_RUNTIME_ENTRY_FEASIBILITY.md)
- [App Intents metadata pipeline Stage 0 audit](swift-abi/APP_INTENTS_STAGE0.md)

The C6 audit found no supported public C/C++ Swift task-entry contract in the inspected Xcode 26.6 interfaces. Exported `swift_task_*` symbols and underscore-prefixed Swift-module bindings are not caller contracts. The compiler-derived async thunk proof does not supply task/context creation, executor, resume, error, or cancellation semantics. Translation and StoreKit 2 async paths remain unsupported until a documented public entry contract or a suitable public Objective-C/C path is established.

The C7 audit observed Xcode 26.6 metadata extraction and a `Metadata.appintents` bundle output, but found no documented stable Rust/C metadata input or processor API. Stage 1 is unsupported on that toolchain; the processor command and metadata format are not treated as public interfaces. Re-audit on Xcode 27.x before making a baseline claim.

## Temporary application escape hatch

A consuming application may temporarily implement a microscopic Swift shim for a framework coverage gap.

That shim is application-owned and must not enter this repository.

When the framework later gains robust Rust-side ABI support, the application shim should be removable.
