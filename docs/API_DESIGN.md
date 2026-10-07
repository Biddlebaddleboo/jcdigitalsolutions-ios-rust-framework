# API Design Rules

## Goal

Make native application development high-level and ergonomic while preserving a Rust-native fast path, cross-platform portability, stable foreign-language compatibility, and minimal runtime cost.

The developer-facing API should feel closer to a platform's preferred high-level language than to raw ABI work.

## Portable capability first

The common API should model developer intent rather than operating-system class names.

Prefer concepts such as:
- secure storage;
- location;
- notifications;
- camera;
- authentication;
- files;
- network requests;
- background work.

Do not expose platform implementation objects in portable signatures.

## Three API levels

Every feature belongs to one of:

1. portable API;
2. portable API plus platform extension;
3. platform-exclusive API.

Do not force platform-exclusive features into fake portability.

## Rust API is native, not a wrapper over C ABI

Rust callers should use direct Rust functions/types.

Do not implement the Rust API by calling exported C ABI functions.

The stable C ABI should adapt to the same underlying Rust implementation.

## High-level does not mean heavyweight

A high-level method should normally compile to:
- pure Rust;
- direct C;
- thin Objective-C/Swift/JNI/native backend operation;
- minimal async/callback adaptation.

Avoid runtime service lookup, reflection, universal boxing, and hidden object graphs.

## Default rule for abstractions

Add an abstraction when it removes meaningful recurring friction or establishes portable semantics.

Good reasons:
- repeated foreign lifetime boilerplate;
- callback/delegate glue;
- Block/JNI/FFI lifecycle;
- platform-thread invariant encoding;
- error conversion;
- common string/data handling;
- a genuinely portable application capability;
- a measured optimization.

Bad reasons:
- cosmetic renaming only;
- creating one framework object for every platform object;
- forcing all platforms into a lowest-common-denominator design;
- making an uncommon native feature wait for a portable wrapper;
- replacing static dispatch with trait-object dispatch without need;
- builders that allocate purely for style.

## Cost transparency

A convenience API should document when it:
- allocates;
- retains/releases;
- copies;
- transcodes;
- boxes;
- locks;
- hops threads;
- stores callback state;
- crosses a language ABI;
- requires a runtime/executor.

Prefer operations that inline to the direct backend call on the Rust path.

## Compact representations

Developer-facing ergonomics must not force bloated internal representations.

For framework-owned values and repeated state:

- choose narrow integer widths when domain bounds are known;
- prefer compact enums/discriminants;
- use bitsets/packed flags for dense boolean/state information;
- prefer contiguous storage and compact IDs where indirection is required;
- split hot and cold fields when it materially improves locality;
- choose AoS/SoA/hybrid layout based on dominant access patterns.

Do not expose needless machine-word-sized fields merely because the host architecture is 64-bit.

Public APIs should preserve semantic clarity even when the internal representation is packed. The Rust API may expose typed newtypes/enums while storing them compactly underneath.

Stable C ABI structs should remain explicit and versionable; internal packed representations may be converted at the ABI boundary instead of freezing a fragile packed layout into the public contract.

Bit packing is first-class, but not mandatory everywhere. Avoid pathological decode cost, unsafe unaligned layout, or obscure representations when a slightly larger layout is faster or substantially safer.

## Ergonomic facade over packed internals

Internal representation must be invisible to normal developers.

For example, an internal capability state may be stored as one packed `u64`, while the Rust API exposes meaningful operations such as:

```rust
state.is_authorized()
state.is_background_allowed()
state.retry_count()
```

rather than making callers manipulate bit positions.

The same principle applies to Python/C++ bindings: expose semantic properties and methods, not implementation masks.

Internally, aggressively consider packing small bounded values into `u32`/`u64` words when it improves cache locality, including:
- multiple booleans/flags;
- small enums;
- bounded counters;
- slot indexes;
- generation counters;
- state-machine phases;
- feature/capability masks.

Keep packing logic centralized in typed helper/newtype implementations. Public APIs should remain readable and stable even if internal bit allocation changes later.

## Dependency-independent public API

Public APIs should describe framework semantics, not the currently chosen implementation crate.

Do not expose third-party dependency types in normal portable/high-level signatures when a framework-owned type can represent the concept cleanly.

Examples:

- expose framework `HttpRequest`/results rather than a specific HTTP crate's request type;
- expose framework-owned error categories rather than a dependency's error enum;
- expose semantic callback/future APIs rather than an executor's task handle;
- expose framework strings/bytes/value types rather than parser/serializer-specific wrappers;
- keep `objc2`, Swift-ABI helper, JNI helper, Python binding, allocator, and generated-binding types behind implementation/platform extension boundaries.

If developers need direct access to an underlying dependency for an advanced case, provide an explicit platform/internal/native escape hatch rather than making that dependency define the ordinary API.

Dependency adapters should be narrow enough that replacing the implementation does not require widespread source changes.

## Stable C ABI design

The ABI should remain intentionally simple:
- fixed-size scalars;
- explicit pointer/length slices;
- opaque handles;
- stable tagged/status values;
- callbacks;
- explicit ownership.

The C ABI is a compatibility surface, not the ideal developer-facing API.

C++ and other language wrappers may provide higher-level source ergonomics over it.

## Python design

Python should expose idiomatic Python:
- objects;
- exceptions;
- awaitables;
- context management where useful.

But Python-specific types/semantics stop at the binding layer.

Avoid chatty native/Python APIs. Prefer one coarse meaningful native operation over repeated per-element crossings.

## Pointer-width-independent semantics

Developer-facing APIs must not expose assumptions that internal references are 64-bit native pointers.

Prefer:
- typed fixed-width IDs;
- semantic handles;
- slices/borrowed references where appropriate;
- explicit bounded counts.

Avoid using `usize` as a public semantic identifier merely because the implementation currently runs on a 64-bit target.

This keeps the API compatible with a future internal implementation using 32-bit cage-relative offsets, compact arena indexes, or compressed handles without changing developer code.

V1 does not need to implement pointer compression.

## no_std-friendly public types

Portable public contracts should prefer `core`/`alloc` compatible concepts.

Avoid requiring:
- `std::fs::File`;
- `std::net`;
- `std::thread`;
- `std::sync::Mutex`;
- boxed `std::error::Error`.

Use framework-owned portable value/error/operation types instead.

## Async

Rust-facing async APIs may expose `Future` without requiring a global executor.

C ABI async should use explicit operations/callbacks/cancellation.

Python adapts operations to awaitables.

Do not make any one language's coroutine runtime the framework's universal runtime.

## Modularity

Users should import only what they need.

Prefer independent capability crates/modules and platform adapters.

An umbrella crate may re-export features, but a minimal capability must not drag unrelated code or system frameworks.

## Native escape

Portable wrappers should provide explicit platform-extension/native access where needed.

Do not make unsupported native functionality impossible merely because the portable layer has not modeled it yet.

## UI

Do not duplicate platform UI semantics in a universal runtime.

Portable UI should remain semantic, while platform backends remain native.

Initial iOS UI is UIKit-based.

## Error design

Rust public APIs should use idiomatic `Result<T, E>`.

The portable error model should preserve enough stable category/code information for C/Python bindings without requiring `std::error::Error`.

Preserve native error detail as optional platform-specific information.

## Strings and bytes

Rust APIs may use idiomatic `&str`, `String`, slices, and owned buffers.

The C ABI must use explicit pointer/length/encoding contracts.

Python conversion occurs at the Python boundary.

Avoid hidden repeated conversions.

## Naming and semantics

Prefer names describing what the developer wants to do rather than mirroring every native selector/method name.

But do not hide meaningful platform semantics such as:
- permissions;
- lifecycle;
- cancellation;
- availability;
- user interaction;
- background limitations.

## Further reference

See `docs/PORTABILITY_AND_ABI.md`.
