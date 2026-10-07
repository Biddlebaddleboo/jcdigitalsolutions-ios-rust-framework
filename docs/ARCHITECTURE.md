# Architecture

## Goal

Provide a high-level native application foundation where Rust is the primary implementation language, platform backends remain native, and the framework contributes as little runtime machinery as possible.

iOS is the first backend, not the architectural definition of the project.

The portable core must support future Android, macOS, Windows, Linux, and Web/WASM backends without redesigning the public capability model.

## Boundary model

The architecture has three domains.

### Portable Rust domain

Keep these as ordinary Rust whenever they do not need a platform object model:

- application state;
- business logic;
- collections and models;
- algorithms;
- parsing;
- protocol logic;
- state machines;
- scheduling decisions;
- capability contracts;
- non-platform-specific utilities.

Prefer:
- static dispatch;
- plain structs/enums;
- compact IDs;
- contiguous storage;
- explicit ownership;
- `core`/`alloc` compatible types where practical.

The portable domain must not expose platform-native object types.

### Platform backend domain

Use native platform objects only where required.

For iOS examples include:
- UIKit views/controllers;
- delegates/protocol objects;
- Foundation types required by API contracts;
- Blocks;
- system framework handles;
- Objective-C runtime objects;
- Swift ABI values only for genuinely Swift-only public APIs.

Future Android/desktop/web backends will have their own native implementation types, but those types must remain behind backend boundaries.

### Foreign-language binding domain

The Rust-native API is the fast path.

Foreign-language compatibility is provided through:
- stable C ABI;
- optional C++ convenience layer;
- optional Python binding;
- future language bindings as needed.

The C ABI and Python layer are siblings over the shared Rust implementation. Rust callers must not be forced through the C ABI.

## Dependency direction

```text
application
   |
   v
portable high-level API
   |
   v
portable capability contracts
   ^
   |
platform backend implements contract
```

Portable crates must not depend on iOS, Android, Windows, Linux, Web, or any other concrete backend.

## Dependency substitution boundary

External dependencies must sit below framework-owned contracts.

A normal dependency path should look like:

```text
portable/high-level API
  -> framework-owned semantic/internal layer
  -> narrow dependency adapter
  -> external crate or generated binding
```

The framework must be able to replace the final adapter/implementation without changing the developer-facing API.

This is especially important for dependencies that may later be replaced for:
- lower runtime overhead;
- smaller binary size;
- better `no_std` compatibility;
- reduced transitive dependencies;
- tighter cache/memory behavior;
- platform coverage;
- ABI control;
- long-term maintenance.

Do not achieve replaceability by imposing universal runtime polymorphism. Dependency selection should normally remain compile-time/static.

Framework-owned interfaces should be as small as possible and reflect the semantics the framework actually needs, not mirror an entire dependency API.

Avoid "wrapper around the whole crate" abstractions. Wrap only the surfaces required to keep architecture independent.

Platform ABI dependencies are a special case: the OS ABI itself cannot be substituted, but the Rust library used to reach it should still be replaceable where practical.

## Platform selection

Prefer compile-time backend selection.

```text
portable API
  -> statically selected backend
  -> native platform API
```

Avoid runtime backend registries, service locators, dependency-injection containers, or boxed trait-object dispatch for normal target selection.

Use dynamic dispatch only when the application genuinely chooses implementations at runtime.

## Portability model

Every capability should be classified as one of:

1. **Portable capability** — same semantic operation across supported platforms.
2. **Portable capability with platform extensions** — common API plus explicit native extensions.
3. **Platform-exclusive capability** — lives under an explicit platform namespace/module.

Do not distort platform-exclusive features into a fake lowest-common-denominator API.

## Default iOS interoperability paths

### Objective-C APIs

```text
Rust
 -> optional thin helper
 -> objc2
 -> Objective-C ABI
 -> Apple framework
```

`objc2` is the default because it provides typed Objective-C messaging, ownership integration, class/protocol support, and framework bindings without a separate managed runtime.

### C APIs

Use direct Rust `extern "C"` bindings or established Rust bindings when they remain thin and correct.

### Swift-only public APIs

Do not add Swift source.

Investigate, in order:

1. public Objective-C exposure;
2. public C/CoreFoundation/Darwin exposure;
3. a missing Rust binding;
4. documented lower-level public primitives;
5. minimal Swift-ABI interoperability.

Swift ABI support must remain optional and capability-scoped.

## Rust-native API and stable C ABI

The Rust API should operate directly on shared Rust implementation types and backend code.

The stable C ABI exists for foreign-language consumers and must use:
- fixed-width primitives;
- pointers and explicit lengths;
- opaque handles;
- explicit ownership/destruction;
- callbacks;
- ABI-stable structs;
- stable status/error codes.

Do not expose Rust compiler layout or platform-native types through the portable ABI.

## Python

Python bindings are optional.

Python concepts must not leak into the core:
- no GIL awareness in portable code;
- no CPython objects in portable contracts;
- no Python allocator assumptions;
- no Python-specific lifecycle requirements.

Convert at the binding edge and keep meaningful work native.

## Modularity

Capabilities must be independently usable.

A small feature should not pull unrelated:
- platform frameworks;
- UI;
- media;
- Swift ABI support;
- Python runtime;
- other capability crates.

An umbrella crate may exist for convenience, but independent capability crates/modules remain authoritative.

Minimal example binaries should validate expected dependencies and binary size.

## no_std direction

The portable core should be designed `no_std`-first.

Long-term tiers:

```text
core only
  -> smallest ABI/value foundations

no_std + alloc
  -> most portable capability models

std
  -> optional backend/convenience functionality
```

Avoid exposing concrete `std` resources such as `std::fs::File`, `std::net` sockets, `std::thread`, or `std::sync::Mutex` in portable public contracts.

Rust `Future` is acceptable because it is a core language abstraction; no executor is mandatory.

## UI architecture

The initial iOS UI backend is UIKit-native.

Do not create a parallel iOS view tree or SwiftUI clone.

At the portable level, model UI/window capabilities semantically. Do not place UIKit types in portable contracts.

Future Android/macOS/Windows/Linux/Web UI backends must not require changing portable ownership or capability semantics.

## Threading

Thread-affinity requirements are platform-specific.

For iOS, prefer a typed main-thread capability such as `objc2::MainThreadMarker`.

Do not invent a global cross-platform "main thread" rule when platform semantics differ.

Avoid global UI locks.

## Callback architecture

Use the minimum object machinery each platform requires.

For iOS target/action/delegate callbacks:
- create narrow Rust-defined Objective-C classes;
- prefer compact/generational callback IDs or static callback functions where appropriate;
- avoid universal boxed closure storage;
- account for reentrancy;
- prevent stale callback invocation;
- prevent retain cycles;
- prevent unwinding across FFI.

At portable/C ABI boundaries, async/callback operations should have explicit lifetime, cancellation, and exactly-once completion rules.

## Ownership

Native platform ownership should map directly to the platform's required semantics.

Do not create a second reference-counting layer around every native object.

Portable ownership must be expressible without platform types.

C ABI ownership is always explicit.

Avoid global handle registries unless stale-handle/cross-thread semantics require indirection.

## Native escape access

Ergonomic wrappers should allow direct platform extension/native access for uncommon features without forcing them into the portable abstraction.

## No mandatory runtime

Normal use must not require:
- global framework initialization;
- a process-wide registry;
- a managed object graph;
- a universal executor;
- a global service locator;
- serialization between internal layers.

High-level API design should come from compile-time structure and thin wrappers, not a heavyweight runtime.

## Further reference

See `docs/PORTABILITY_AND_ABI.md` for the detailed ABI, modularity, language-binding, and no_std rules.
