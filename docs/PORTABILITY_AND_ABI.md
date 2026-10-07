# Portability, ABI, Modularity, and no_std Architecture

## Purpose

This document defines how the framework must remain high-level for developers while preserving:
- a Rust-native fast path;
- stable C ABI compatibility;
- optional Python bindings;
- future Android/macOS/Windows/Linux/Web backends;
- fine-grained modularity;
- little-to-no framework runtime overhead;
- straightforward future migration to `no_std`.

## 1. Architectural identity

The project is a platform-agnostic native application framework implemented primarily in Rust.

iOS is the first backend, not the definition of the framework.

The public portable layer should model application capabilities rather than Apple-specific classes or Android-specific objects.

## 2. Layering

```text
Application code
  |
  +-- Rust API ------------------------------+
  |                                          |
  +-- C/C++/other C-ABI bindings ------------+--> shared Rust capability implementation
  |                                          |
  +-- Python bindings -----------------------+
                                             |
                                             v
                                   platform-agnostic core
                                             |
                        +--------------------+--------------------+
                        |                    |                    |
                        v                    v                    v
                       iOS                Android              Desktop/Web
                        |                    |                    |
             Rust/C/objc2/Swift ABI   Rust/NDK/JNI/Binder   native/Web APIs
```

Rust must not call through the C ABI just because a C ABI exists.

The C ABI and Python bindings are sibling frontends over the same Rust implementation.

## 3. Rust-native fast path

Rust is the canonical implementation path.

Portable Rust wrappers should be:
- thin;
- inlineable;
- statically dispatched;
- monomorphizable;
- compatible with LTO/dead-code elimination.

Compile-time backend selection is preferred.

Avoid:
- `Box<dyn Backend>` for normal platform selection;
- runtime service lookup;
- global registries;
- reflection;
- string-based dispatch.

A portable call should ideally optimize to the direct backend call.

## 4. Stable C ABI

The stable ABI exists so any C-capable language can consume the framework.

Use:
- `u8/u16/u32/u64` / fixed-width integer equivalents;
- pointers;
- explicit lengths;
- ABI-stable structs;
- explicit enum/status representations;
- opaque handles;
- callbacks;
- explicit create/destroy/transfer rules;
- version fields for extensible structs where needed.

Never expose unspecified Rust layout.

Do not expose:
- `Vec<T>`;
- Rust `String`;
- Rust references;
- `Box<T>`;
- `Arc<T>`;
- trait objects;
- platform-specific ownership wrappers;
- compiler-private Rust ABI.

The C ABI should remain boring and stable even if internal Rust architecture changes.

## 5. Python bindings

Python is optional.

Python bindings should:
- expose idiomatic Python APIs;
- provide awaitables for asynchronous operations when appropriate;
- convert Python values at the outer edge;
- enter native Rust once per meaningful operation;
- keep loops/heavy computation native;
- avoid high-frequency Python callbacks unless the feature inherently requires them.

Python runtime overhead is not framework overhead. Do not burden Rust/C users with CPython dependencies.

## 6. Portable capabilities versus platform extensions

Use three classes.

### Portable capability

Same developer-facing semantic operation across platforms.

Examples:
- HTTP;
- files;
- preferences;
- secure storage;
- location;
- notifications;
- camera;
- Bluetooth;
- clipboard;
- authentication;
- background work.

### Portable capability with platform extension

Common operations stay portable; uncommon platform-specific features remain accessible.

Example:

```text
Camera
  +-- portable capture/configuration
  +-- ios extension
  +-- android extension
```

### Platform-exclusive capability

Do not invent fake portability.

Examples:
- iOS ActivityKit;
- Android Play Integrity;
- Windows Jump Lists;
- Web Service Worker controls.

Keep these in explicit platform namespaces/modules.

## 7. Platform type firewall

Portable public APIs must not expose platform-native implementation types.

Forbidden in portable contracts:
- Objective-C pointers/classes;
- UIKit/AppKit types;
- JNI/JVM objects;
- Android Activity/Context;
- Win32/COM/WinRT handles;
- X11/Wayland types;
- DOM/JS objects;
- Swift metadata/types.

Native handles may be available only through platform extension/escape APIs.

## 8. Modularity

Capabilities must be independently importable.

Preferred shape:

```text
core/
  framework-core
  framework-alloc
  framework-abi
  framework-async

capabilities/
  framework-files
  framework-preferences
  framework-secure-storage
  framework-network
  framework-location
  framework-notifications
  framework-camera
  framework-bluetooth
  ...

platform/
  ios/
    ios-files
    ios-location
    ios-notifications
    ...
  android/
  macos/
  windows/
  linux/
  web/

interop/
  swift-abi-core
  swift-abi-async
  swift-abi-collections
  ...

bindings/
  c/
  cpp/
  python/
```

Exact crate names may change, but the dependency direction must remain.

## 9. No mandatory umbrella runtime

Do not require:

```text
Framework::initialize()
global runtime
global service manager
global object graph
mandatory executor
```

Most capabilities should be independently usable.

An optional umbrella crate may re-export capabilities, but it must not make them interdependent.

## 10. Linkage discipline

A small capability must not drag in unrelated frameworks.

Examples:

```text
secure storage
  -> Security / platform equivalent
  -> no camera
  -> no UIKit unless genuinely required
  -> no Swift ABI unless required
```

```text
location
  -> location backend
  -> no AVFoundation
  -> no StoreKit
  -> no Python
```

Add minimal example binaries and inspect:
- final linked frameworks/libraries;
- Mach-O/ELF/PE imports;
- dependency graph;
- binary size.

Treat accidental linkage as a regression.

## 11. no_std migration

Portable foundations should be written `no_std`-first even while the project initially builds with `std`.

Prefer:
- `core`;
- `alloc`;
- framework-owned abstractions.

Avoid public dependence on:
- `std::thread`;
- `std::sync::Mutex`;
- `std::fs::File`;
- `std::net`;
- concrete `std::io` types;
- `Box<dyn std::error::Error>`.

Long-term tiers:

```text
core-only
  -> ABI primitives / tiny value types

no_std + alloc
  -> most portable capability models

std
  -> optional platform/backend conveniences
```

The long-term target for most portable code is `no_std + alloc`, not allocator-free code everywhere.

## 12. Async model

Do not make Tokio, async-std, Swift concurrency, Kotlin coroutines, Python asyncio, or any other runtime the universal internal model.

The portable operation model must be runtime-neutral.

At the Rust API:
- expose `Future` when useful;
- allow callback-oriented internals;
- avoid a mandatory executor.

At the C ABI:
- operation handle;
- completion callback;
- cancellation;
- explicit lifetime.

Bindings adapt:
- Rust -> Future;
- Python -> awaitable;
- C -> callback;
- C++ -> callback/future wrapper.

## 13. Ownership

Rust owns Rust state.

Platform backends own/retain native platform handles only as required.

Foreign ABI ownership is explicit.

Avoid:
- duplicate reference-counting layers;
- universal handle registry;
- always-boxed objects.

Opaque pointers can often directly point to Rust-owned state. Use generation/registry mechanisms only when semantics require stale-handle detection or indirect identity.

## 14. Error model

Portable errors must not require `std::error::Error`.

Use stable enums/codes plus optional platform detail.

C ABI:
- stable status code;
- optional owned error/detail handle.

Rust:
- idiomatic `Result<T, E>`.

Python:
- idiomatic exception mapping.

Preserve native detail without making platform error types part of portable contracts.

## 15. Strings and bytes

Portable Rust APIs may use idiomatic Rust strings/slices.

Stable C ABI must use explicit pointer + length + encoding.

Python conversion belongs at the Python edge.

Avoid repeated transcoding and copies.

Borrow where safe; own where lifetime requires it.

## 16. UI policy

The common UI API may be high-level, but must not become a framework runtime or virtual DOM by default.

Initial iOS backend:
- UIKit-native.

Future backends:
- Android native UI;
- AppKit/macOS native UI;
- Windows native UI;
- Linux native backend;
- Web/DOM.

Do not require one rendering model to emulate all platforms.

Portable UI concepts should stay semantic and allow platform extensions.

## 17. Performance invariants

### Rust

Target:

```text
high-level portable API
  -> inline/static dispatch
  -> backend
  -> native platform call
```

The portable abstraction should often disappear after optimization.

### C/C++

Expected incremental cost:
- normal C ABI call;
- required representation conversion only.

### Python

Expected:
- Python-to-native call overhead;
- CPython runtime;
- native work after the boundary.

Framework implementation should avoid multiplying crossings.

## 18. Swift ABI modularity

Swift ABI support belongs only to iOS/macOS capabilities that need it.

Do not make it part of the portable core.

Split it finely enough that:
- StoreKit does not require App Intents metadata support;
- Translation does not pull protocol/type-definition machinery it does not need;
- native-only apps link none of the project's Swift ABI subsystem.

## 19. Platform backend dependency rule

Dependency direction:

```text
portable core/capability contract
        ^
        |
platform backend implements it
```

Never:

```text
portable core -> iOS backend
portable core -> Android backend
portable core -> Windows backend
```

The portable layer must compile without any specific platform backend.

## 20. Cross-platform semantic compatibility

The developer-facing portable API should remain source-compatible across platforms where semantics match.

Do not promise identical system behavior when OS policies differ.

Document:
- unsupported capabilities;
- degraded behavior;
- platform-specific permission/lifecycle constraints;
- optional extensions.

Semantic portability is more important than pretending implementation details are identical.

## 21. Build/CI invariants

Future CI should include:

- portable core build with no platform backend;
- `--no-default-features` checks;
- iOS backend;
- Android backend when added;
- desktop/web backend builds as added;
- minimal capability examples;
- C ABI compile/link tests;
- ABI compatibility tests;
- Python binding import/tests when enabled;
- unexpected dependency/linkage checks;
- binary-size tracking;
- assembly/codegen checks for critical zero-cost wrappers.

## 22. Design review questions

Before adding a capability, ask:

1. Is the concept portable?
2. Does it belong in a platform extension instead?
3. Can the Rust path stay direct and statically dispatched?
4. Does the C ABI need a new stable primitive?
5. Can the portable contract remain `no_std + alloc` compatible?
6. Does this capability accidentally link unrelated modules?
7. Does it introduce a new mandatory runtime/global singleton/executor?
8. Can Python bind it without pushing Python concepts into core?
9. Does it preserve native escape access?
10. What exact additional runtime cost does the abstraction add?

## Final rule

The framework should be high-level at the source level and low-level at runtime.

Portability, language interoperability, and ergonomics must be achieved through compile-time structure and thin ABI adapters—not through a heavyweight universal runtime.
