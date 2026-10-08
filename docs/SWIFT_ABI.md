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

## Temporary application escape hatch

A consuming application may temporarily implement a microscopic Swift shim for a framework coverage gap.

That shim is application-owned and must not enter this repository.

When the framework later gains robust Rust-side ABI support, the application shim should be removable.
