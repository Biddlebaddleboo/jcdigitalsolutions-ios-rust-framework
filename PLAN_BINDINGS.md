# PLAN_BINDINGS.md — Workstream F: Stable C/C++/Python Binding Surfaces

## Objective

Expose the completed Rust framework to foreign languages without making the foreign ABI the Rust implementation path.

## Dependencies

Requires A foundation.
Capability-specific bindings start only after the corresponding D public contract is stable.

## Execution decomposition

Start with `PLAN_BINDINGS_CORE.md` (F1), which freezes and validates the foundational C ABI from A without waiting for capability crates. Add capability-specific headers only in later named slices after their D contracts are stable. C++ and Python remain optional and must not block the C ABI.

## Write scope

- `bindings/c/**`
- `bindings/cpp/**`
- `bindings/python/**`
- `examples/c-minimal/**`
- ABI tests and binding docs

## C ABI

Create a stable modular C ABI using `framework-abi`.

Requirements:
- fixed-width scalars;
- pointer+length strings/bytes;
- versioned structs;
- opaque handles;
- explicit ownership/destruction;
- explicit callbacks;
- operation handles/cancellation for async;
- stable status/error codes;
- capability-scoped headers/libraries so one small API does not force unrelated linkage.

Do not expose:
- Rust references;
- `Vec`, `String`, `Box`, `Arc`;
- trait objects;
- Rust enum/layout without specified repr;
- `objc2::Retained`;
- Swift metadata/types;
- platform-native object types in portable contracts.

Platform-specific C escape APIs may expose opaque native handles where explicitly documented.

## C header generation

Choose the smallest dependable strategy:
- hand-maintained header for small stable core if safer;
- or cbindgen-like generation if dependency/tooling value justifies it.

Generated header output must be diff/tested for ABI changes.

## C++ layer

Optional V1 convenience layer:
- header-only RAII where possible;
- typed wrappers over C ABI;
- no runtime registry;
- no second implementation;
- no hidden allocation beyond semantics.

Do not confuse Swift-generated C++ ABI oracle headers with the framework's public C++ API.

## Python

Python is optional and should not block V1 native completion.

If implemented in V1:
- top-level binding only;
- idiomatic Python objects/exceptions/awaitables;
- heavy loops remain Rust;
- avoid chatty per-element crossings;
- no Python dependency for Rust/C builds;
- Python-specific ownership/GIL stops at binding edge.

Select PyO3 or alternative only after dependency/runtime/build-cost review. Keep Python binding machinery replaceable.

## Async ABI

C:
```text
start(...)
 -> operation handle
 -> callback exactly once
cancel(handle)
destroy(handle)
```

Document whether cancel guarantees prevention or merely requests cancellation.

C++ may wrap operation handle in RAII/future-like convenience without changing underlying semantics.

Python may adapt to an awaitable/future while preserving native cancellation rules.

## ABI compatibility tests

Test:
- `sizeof`/`alignof` known structs;
- symbol exports;
- header compile from C11 and C++;
- ownership create/destroy;
- callback signatures;
- error mapping;
- backward-compatible struct extension;
- no panic across boundary.

Use an ABI manifest/version record.

## Linkage tests

C minimal consumers for:
- core only;
- one storage capability;
- one networking capability;
- one UI capability.

Verify no unrelated frameworks/bindings.

## Non-goals

- Rust API never calls the C ABI;
- no universal serialized message protocol;
- no COM-like object model;
- no Python runtime in core;
- no freezing internal packed layout into public ABI.

## Handoff

Report:
- exported ABI version;
- header modules;
- ownership table;
- async rules;
- compatibility tests;
- optional bindings actually completed.
