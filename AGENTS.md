# AGENTS.md

## Purpose

This repository builds a reusable, high-level, platform-agnostic native application framework whose primary implementation language and primary fast-path API are Rust.

The framework must support iOS first while being architected from the beginning for Android, macOS, Windows, Linux, and Web/WASM backends. It must also expose a stable C ABI so C-compatible languages can consume the framework, and support an optional Python binding layer.

The framework is intentionally not a Flutter/React-Native-style runtime, not a virtual DOM, not a custom renderer, not a Swift wrapper around a Rust core, and not a language VM. Platform-native objects remain native platform objects. The framework should add as close to zero incremental runtime cost as practical.

These rules are architectural invariants. Do not weaken them for convenience.

## Hard rules

### Rust-native fast path

- Rust is the primary implementation language.
- Rust applications must use a native Rust API that calls shared Rust implementation code directly.
- Do not force Rust callers through the exported C ABI.
- The C ABI is a foreign-language compatibility boundary, not the internal architecture.
- Preserve opportunities for monomorphization, inlining, constant propagation, dead-code elimination, and LTO on the Rust path.
- Prefer static dispatch and compile-time backend selection over trait-object dispatch or service lookup.

C, C++, Objective-C, Objective-C++, or assembly may be used only when a concrete ABI, toolchain, or platform requirement makes that the smallest correct solution. Keep such code microscopic and document why Rust alone is insufficient.

### Stable C ABI for foreign-language compatibility

The framework must be consumable by languages that can call a C ABI.

The stable ABI must use language-neutral primitives such as:
- fixed-width integers;
- pointers;
- explicit lengths;
- opaque handles/pointers;
- explicit status/error values;
- explicit ownership/destruction;
- callbacks and completion functions;
- ABI-versioned structs where needed.

Do not expose these across the stable ABI:
- Rust `Vec`, `String`, `Box`, `Arc`, references, trait objects, enums without a defined representation, or compiler-specific Rust layout;
- Objective-C ownership wrappers such as `Retained<T>`;
- Swift ABI implementation details;
- platform-specific object types in portable contracts.

C ABI wrappers should call the same Rust core used by the Rust-native API. They must not define a second implementation.

### Optional Python binding

Python support must be an optional top-level binding layer over the same shared core.

- The portable core must not depend on CPython, the GIL, Python objects, Python allocation semantics, or Python packaging.
- Python's interpreter/runtime cost is an opt-in cost paid only by Python applications.
- Keep expensive/high-frequency work native; avoid repeated fine-grained Python/native crossings.
- Python bindings should provide idiomatic Python objects/awaitables while translating to language-neutral native operations underneath.
- Python support must not increase runtime cost, binary dependencies, or initialization requirements for Rust/C users who do not enable it.

### Platform-agnostic portable core

The portable layer must model application capabilities, not operating-system APIs.

Portable public contracts must not contain:
- `UIView`, `UIViewController`, `NSObject`, or other Apple types;
- Android `Activity`, JNI objects, Binder implementation details, or Java/Kotlin types;
- Windows `HWND`, COM/WinRT implementation types;
- Wayland/X11 handles;
- JavaScript/DOM/WebAssembly binding types.

Expose concepts such as:
- application/window/screen;
- files/preferences/secure storage;
- HTTP/networking;
- notifications;
- location;
- camera/audio;
- Bluetooth;
- clipboard/share;
- authentication;
- permissions;
- background work;
- platform capabilities that can be described semantically.

Do not force fake portability. Support three levels explicitly:
1. fully portable capability;
2. portable capability with platform-specific extensions;
3. genuinely platform-exclusive capability.

A platform-specific capability may live under an `ios::`, `android::`, `windows::`, `web::`, etc. namespace/module rather than being distorted into a lowest-common-denominator API.

### Platform backends must be replaceable and statically selected

Architect the framework so backends can be added for:
- iOS;
- Android;
- macOS;
- Windows;
- Linux;
- Web/WASM;
- future platforms without redesigning the portable core.

Prefer compile-time target selection:

```text
portable API
  -> statically selected backend
  -> native platform API
```

Do not require:
- runtime backend discovery;
- dependency injection containers;
- global service registries;
- string-based capability lookup;
- boxed `dyn Backend` for ordinary platform selection.

Use dynamic dispatch only where the application genuinely needs runtime-selected implementations.

### Underlying dependencies must be replaceable

Third-party crates, generated bindings, helper libraries, allocators, parsers, executors, and other implementation dependencies are implementation choices, not permanent public architecture.

Design every important external dependency so it can be replaced later by an in-house implementation without forcing application-level API changes.

Rules:

- Do not leak third-party dependency types into the portable public API unless that type is itself an unavoidable platform ABI type.
- Keep dependency-specific types and behavior behind narrow internal modules/adapters.
- Prefer framework-owned semantic types at public and cross-capability boundaries.
- Keep conversion to/from dependency-native types at the smallest practical boundary.
- Do not let a dependency define framework ownership, error, async, allocation, serialization, or lifecycle semantics globally unless that dependency is itself the platform ABI being wrapped.
- Avoid dependency-specific macros/code generation becoming required throughout unrelated framework code.
- Centralize dependency-specific unsafe assumptions and feature flags.
- Make dependency removal/replacement testable through focused adapter/backend tests.
- Record which behavior is relied upon so a replacement implementation can preserve semantics.

Replaceability must not introduce a runtime abstraction tax.

Prefer:

```text
public API
  -> framework-owned internal contract
  -> compile-time-selected implementation adapter
  -> current dependency
```

over:

```text
public API
  -> boxed dynamic dependency interface
  -> runtime registry
  -> dependency
```

Use generics, sealed/internal traits, module substitution, Cargo features, cfg selection, or build-time backend choice where they preserve static dispatch and inlining.

Examples of dependencies that must remain replaceable in principle include:

- `objc2` / `block2`;
- generated Apple bindings;
- Swift ABI thunk implementation;
- allocators/arenas;
- async helpers;
- parsing/encoding libraries;
- networking helper layers;
- cryptographic/helper crates where API semantics permit;
- Python binding machinery;
- C header/binding generators;
- platform-specific utility crates.

This does not mean reimplementing mature dependencies prematurely. Use the best current dependency, but architect so replacing it later is a bounded internal change.

### Fine-grained modularity

A consumer that needs one small capability must not have to import or link a large unrelated framework.

- Split capabilities into independently usable crates/modules.
- Keep platform implementations capability-scoped where practical.
- Swift ABI support must be opt-in and linked only by capabilities that need it.
- Python bindings must be opt-in.
- UI must not be pulled in by storage/security/network-only users.
- Camera/media must not be pulled in by simple preference or Keychain users.
- Avoid a mandatory umbrella runtime library.
- An umbrella convenience crate may re-export feature crates, but independent crates must remain usable directly.

Cargo feature flags are useful, but do not rely on one giant crate with deeply entangled conditional compilation when separate crates provide clearer boundaries.

Add CI/linkage tests that detect unexpected platform frameworks, large dependency growth, or binary-size regressions in minimal examples.

### Future no_std migration is a first-class constraint

Design the portable core and public capability contracts so migration to `no_std` is straightforward.

- Prefer `core` and `alloc` types in portable code where practical.
- Treat `std` as an optional implementation convenience, not an architectural requirement.
- Aim for `no_std + alloc` as the default long-term portability target for most portable layers.
- Very small foundational crates should be able to become true `core`-only where practical.
- Do not bake `std::thread`, `std::sync::Mutex`, `std::fs::File`, `std::net`, concrete `std::io` types, or `std::error::Error` ownership into portable public APIs.
- Rust `Future` is allowed because it is a core language abstraction; no global executor may be required.
- Build core crates with `--no-default-features` in CI as soon as corresponding crate structure exists.

Platform backends may use `std` initially where needed, but `std` dependencies must stay out of portable contracts.

### No mandatory framework runtime

Normal use must not require a process-wide framework initialization step.

Do not introduce, unless a concrete requirement proves necessary:
- a garbage collector;
- a framework-wide `Arc`/`Rc` ownership model;
- a global object registry for every object;
- a dependency injection container;
- a global service locator;
- a framework-wide task scheduler;
- a mandatory async runtime such as Tokio;
- a global UI-state mutex;
- JSON/serialization between framework layers;
- IPC between framework layers;
- a virtual UI tree;
- a reconciliation engine;
- a custom renderer for ordinary native views;
- string-based method routing;
- reflection for ordinary access;
- boxed dynamic callbacks for every event;
- duplicate platform object models.

Opaque C handles should be direct opaque pointers/owned Rust objects when safe and practical. Use registries only where stale-handle protection, callback identity, cross-thread lifetime, or platform semantics require them.

### High-level API, low-level implementation

The developer-facing API should be high-level and ergonomic, roughly comparable to the convenience level a platform's preferred high-level language provides.

High-level does not mean heavyweight.

A convenience layer should compile down to:
- pure Rust;
- a direct C call;
- a thin platform ABI call;
- or a minimal native callback/async adapter.

Common application code should not need to manipulate selectors, JNI details, Swift metadata, raw handles, ownership markers, or foreign callback trampolines.

Preserve platform-native escape hatches for capabilities not yet wrapped.

### Zero Swift source in the framework

- No `.swift` files may exist anywhere in this repository.
- The build must not generate Swift source files.
- Do not solve missing Apple API coverage by adding a Swift bridge to this repository.
- Apple frameworks may themselves be implemented in Swift; that does not violate this rule.
- If a public Apple API is genuinely Swift-only, first investigate a Rust implementation of the minimum Swift ABI interoperability required to call it.
- A consuming application may temporarily own a microscopic Swift shim for an unsupported API, but such source must remain outside this framework repository.

### App Store compliance is mandatory

Only public, supported Apple interfaces may be used for shipping functionality.

Do not use:
- private frameworks;
- private selectors or symbols;
- undocumented system-daemon protocols;
- private entitlements;
- entitlement bypasses;
- reverse-engineered private APIs;
- behavior intended to evade App Review;
- downloaded executable code contrary to App Store rules.

Technical feasibility does not override App Store compliance.

For unusual low-level integrations, document the public API ultimately being used, availability, entitlement requirements, and App Store implications.

### objc2 is the default Objective-C interoperability layer

Use `objc2` and its framework crates for public Objective-C APIs unless there is a measured or correctness-based reason not to.

Do not reimplement `objc_msgSend`, Objective-C ownership machinery, Blocks, or framework bindings simply because lower-level code looks faster.

Direct Objective-C ABI calls or narrowly scoped assembly are allowed only when at least one of the following is true:
1. the required ABI surface is not correctly expressible through current bindings;
2. generated-code inspection shows avoidable incremental overhead;
3. a benchmark demonstrates a meaningful improvement.

Any bypass must preserve Objective-C ownership, calling conventions, error semantics, and App Store compliance.

### Optimize the portable and Rust sides as Rust, not as platform objects

Only objects that must participate in a platform object model should be native foreign objects.

Keep application/framework-owned computation in ordinary Rust representations when possible:
- structs and enums;
- slices and `Vec`;
- compact/generational IDs;
- static dispatch and generics;
- arenas/slabs when justified;
- cache-friendly layouts;
- SIMD or architecture-specific code when measured.

Do not turn ordinary application state into `NSObject`, Java objects, COM objects, DOM objects, or other foreign runtime objects.

### Cache locality, compact data, and bit packing are first-class requirements

The portable core, capability state, callback state, registries, queues, and other framework-owned data must be designed with CPU cache behavior and memory density in mind.

Default rules:

- Prefer the smallest integer width that safely represents the required domain and realistic future scale.
- Prefer compact enums and flags over machine-word-sized values when the domain is bounded.
- Treat structure padding, alignment, cache-line footprint, and element stride as design inputs.
- Prefer contiguous storage and cache-friendly iteration over pointer-heavy object graphs.
- Prefer compact/generational IDs over pointer-sized handles when indirect identity is required and the capacity bound permits it.
- Use structure-of-arrays, array-of-structs, or hybrid layouts according to the dominant access pattern rather than by habit.
- Avoid storing redundant derived state when it materially increases working-set size.
- Separate cold/rarely used fields from hot frequently accessed state when doing so reduces hot-structure footprint.
- Avoid per-element heap allocation in dense collections unless ownership or semantics require it.

Bit packing is a first-class tool, not a forbidden micro-optimization.

Use bitsets, packed flags, tagged integers, narrow discriminants, compact indexes, and packed state fields when they materially reduce footprint or improve locality without violating correctness.

However:

- Do not use packed/unaligned representations blindly.
- Avoid `repr(packed)` when ordinary explicit bit fields/masks or field reordering achieve the same result safely.
- Account for architectures where unaligned loads are slower or constrained.
- Do not compress values so aggressively that every access requires expensive decode work unless measurements justify the tradeoff.
- Public/stable C ABI structs must prioritize ABI clarity and compatibility over internal packing tricks; keep packed internal representations behind conversion boundaries when necessary.
- Document numeric capacity/range invariants for narrow IDs/counters.
- Overflow, sentinel values, generation wraparound, and invalid bit patterns must have explicit behavior.

For hot or repeated structures, inspect `size_of`, alignment, padding, stride, and cache-line occupancy. Benchmark representative traversal/mutation patterns when alternative layouts have meaningful tradeoffs.

The objective is not "smallest possible struct at any cost." The objective is the smallest representation that preserves correctness, portability, predictable access, and good CPU behavior.

Developer-facing ergonomics must be independent from internal packing.

- Public Rust, C++, Python, and other high-level bindings should expose descriptive fields, typed enums/newtypes, methods, builders, and capability-oriented APIs even when the internal storage is a packed integer or bitset.
- Never require application developers to manually shift/mask bits, manage sentinel encodings, decode packed IDs, or understand cache-layout choices for ordinary use.
- It is encouraged to pack multiple small flags, bounded counters, discriminants, indexes, generations, and other compact state into one or a few `u32`/`u64` words when that improves locality and remains cheap to access.
- Internal encode/decode helpers should be small, typed, inlineable, and centralized so packing does not spread representation knowledge through the codebase.
- The public semantic API is stable; internal bit allocation/layout may change without breaking users unless it crosses an explicitly versioned external ABI or serialized format.
- Prefer typed getters/setters and newtypes that compile down to masks/shifts over exposing the packed storage word itself.

### Abstraction must justify itself

A framework abstraction should exist when it:
- removes recurring ABI/lifetime boilerplate;
- enforces an important invariant;
- materially improves developer ergonomics;
- exposes a genuinely portable concept;
- enables a measurable optimization.

Prefer thin, inlineable wrappers. Preserve access to underlying native capabilities so uncommon platform features do not wait for framework wrapper coverage.

Do not add abstraction solely to rename platform APIs.

### UIKit is the initial iOS backend UI

The initial iOS backend is UIKit-native.

Do not implement a SwiftUI clone, virtual DOM, custom text engine, custom scrolling system, custom accessibility tree, or custom renderer in the first architecture.

This rule does not make the portable core iOS-specific. UIKit is one backend implementation of portable UI/window concepts and of iOS-specific extensions.

A future Android/desktop/web UI backend must not require changing the portable core's fundamental ownership or capability model.

## Ownership and lifetime rules

Use Objective-C ownership semantics directly in the iOS backend:
- `Retained<T>` for owned strong Objective-C references;
- borrowed references where ownership is unnecessary;
- `Weak<T>` where weak semantics are required;
- `Allocated<T>` during initialization where appropriate.

Do not wrap `Retained<T>` in `Arc` or `Rc` by default.

Portable API ownership must be expressible without Objective-C-specific types.

C ABI ownership must be explicit: every created/owned object must have documented destruction or transfer semantics.

Avoid unnecessary retain/release traffic. Extra framework-generated retain/release, clones, reference-count layers, or wrapper allocations are optimization bugs unless justified.

## Main-thread and threading rules

Platform-specific UI thread rules belong in platform backends.

For iOS, prefer `objc2::MainThreadMarker` or an equally zero-cost typed capability.

Do not make a global "main thread" abstraction that assumes every platform has identical UI-thread semantics.

The portable API may encode "UI-thread-affine" or "platform-thread-affine" requirements semantically where necessary, with each backend enforcing the correct native rule.

Do not default to `Arc<Mutex<...>>` for UI or portable application state.

Handle reentrancy explicitly. Never fabricate permanent mutable references to global state that can alias during synchronous callbacks.

## Objective-C classes, delegates, and target/action

Use Rust-defined Objective-C classes through `objc2::define_class!` where appropriate in the iOS backend.

Create Objective-C bridge objects only where Apple APIs require an Objective-C object.

Keep bridge objects narrow.

For common callbacks, prefer compact IDs, statically known functions, or similarly cheap dispatch over `Arc<Mutex<Box<dyn FnMut(...)>>>`.

Handle stale callback IDs, teardown, reentrancy, deallocation, and retain cycles.

A Rust panic must never unwind through any foreign ABI boundary.

## Blocks

Use `block2` for Apple Blocks.

Distinguish escaping from nonescaping callbacks. Avoid heap allocation or retained closure state when the Apple API and Rust callback shape do not require it.

Audit block capture graphs for retain cycles.

## Strings and bytes

Portable APIs should distinguish borrowed and owned data where the language permits it.

Do not eagerly convert every platform string/data value into a new Rust allocation.

For iOS, distinguish:
- borrowed native strings;
- retained native strings;
- temporary strings;
- static strings;
- owned Rust strings;
- borrowed native bytes;
- retained `NSData`;
- borrowed byte slices;
- owned `Vec<u8>`.

For C ABI bindings, use explicit pointer/length/encoding contracts and ownership rules.

For Python, convert at the outer binding edge and avoid repeated boundary conversions.

Measure conversion and copy costs before adding convenience conversions to hot APIs.

## Swift-only public APIs

The framework contains no Swift source.

Escalation order:
1. public Objective-C interface via `objc2`;
2. public C/CoreFoundation/Darwin interface via Rust FFI;
3. add/generate a missing public binding;
4. implement equivalent convenience behavior over documented lower-level public APIs;
5. for a genuinely public Swift-only API, implement the minimum required Swift ABI interoperability;
6. leave the API unsupported until a robust compliant path exists.

Do not build a general Swift compiler/runtime clone speculatively.

Swift ABI crates/modules must remain optional and capability-scoped. A user who does not use a Swift-only Apple feature must not pull in unrelated Swift-ABI machinery added by this project.

## Unsafe Rust

Unsafe is acceptable and expected in native platform backends and ABI layers.

Prefer small unsafe cores with safe or narrowly unsafe callers.

Every unsafe block or subsystem must make clear:
- calling-convention assumptions;
- ownership;
- lifetime;
- aliasing;
- thread constraints;
- initialization state;
- unwind behavior;
- pointer validity.

Prefer `#![deny(unsafe_op_in_unsafe_fn)]` where practical.

Unsafe is not a performance feature by itself.

## Assembly

Assembly is allowed for:
- ABI thunks;
- measured hot paths;
- operations where compiler output is demonstrably inferior.

Before adding assembly:
1. inspect optimized compiler output;
2. benchmark the existing implementation;
3. prove the assembly changes the relevant cost;
4. document ABI/clobber assumptions;
5. provide a safe/correct fallback where needed.

Do not use assembly merely to appear lower level.

## Performance standard

### Portable abstraction

For Rust callers, high-level portable wrappers and compile-time backend selection should add no measurable runtime dispatch or allocation in ordinary simple operations when they can be inlined/static-dispatched.

### Foreign-language ABI

For C/C++ callers, the additional cost should normally be limited to the ordinary C ABI call and any representation conversion inherently required by the language boundary.

### Python

Do not claim Python itself has zero runtime overhead. The target is minimal framework-added overhead beyond CPython and the native operation.

### Apple boundary

For an Objective-C Apple API, Rust + `objc2` should approach the cost of equivalent Objective-C as closely as practical.

If an Objective-C call ultimately requires `objc_msgSend`, that dispatch is an Apple/Objective-C cost, not a Rust penalty.

### Framework/application computation

Code not required to use a foreign platform object model should target ordinary optimized Rust/C++ performance.

Measure:
- launch time;
- RSS;
- allocations;
- retain/release traffic;
- callback dispatch;
- string/data conversion;
- networking setup/completion/body copies;
- FFI crossings;
- portable-wrapper overhead;
- binary size;
- linked dependency/framework set.

Performance claims must be backed by reproducible measurements.

## Benchmark comparison

For iOS UIKit work, compare at least:
- Objective-C/UIKit;
- Swift/UIKit;
- native Rust/iOS through this framework.

Also compare:
- Rust portable API vs direct Rust backend API;
- Rust native path vs C ABI path when both exist;
- minimal capability binaries to detect unwanted linkage.

SwiftUI may be included as an architectural comparison, but do not treat it as the zero-overhead UIKit baseline.

Inspect optimized assembly for important ABI hot paths.

## API design

The framework should make common cross-platform native application development high-level without constructing a second runtime.

Good abstraction:
- expresses a portable capability;
- hides repetitive selector/delegate/JNI/FFI/lifetime boilerplate;
- stays thin;
- statically dispatches where practical;
- exposes platform extension/native access;
- adds no hidden allocation/copy/lock in the hot path.

Bad abstraction:
- mirrors each platform into a giant duplicate object tree;
- forces every capability into a lowest-common-denominator model;
- requires runtime reconciliation;
- boxes every operation;
- serializes values between internal layers;
- prevents native escape;
- forces all applications to link all capabilities.

## Networking and async

Do not require Tokio or another general-purpose executor.

The portable async contract should be callback/future friendly and executor-neutral.

At a stable C ABI boundary, use explicit operation handles/callbacks/cancellation rather than exposing Rust Future, Swift async internals, Python coroutine internals, or Kotlin coroutine internals.

Language bindings may adapt the same operation to:
- Rust `Future`;
- Python awaitable;
- C callback;
- C++ future/callback;
- future Android-language bindings.

Preserve cancellation, error detail, exactly-once completion, and data ownership.

## Repository/planning workflow

Before implementation:
- read `PLAN.md` first;
- read any assigned `PLAN_*.md`;
- verify the latest `main`;
- stay within the named files/symbols unless compilation, tests, moved code, or correctness requires expansion;
- do not silently redesign architecture when a plan conflicts with current code; report the contradiction.

Executors should report:
- changed files;
- commit SHA;
- tests/benchmarks run;
- deviations from plan;
- unresolved assumptions.

The planning files are temporary execution handoff artifacts and should normally be deleted before the final implementation commit once the implementation has been independently validated.

## Before merging framework changes

Check:
- no `.swift` file was added or generated;
- only public supported platform APIs are used;
- portable APIs contain no platform-native types;
- Rust callers do not unnecessarily route through the C ABI;
- optional bindings/backends do not become mandatory dependencies;
- a minimal capability does not pull unrelated capability crates/frameworks;
- no new portable-core `std` dependency was introduced without justification;
- no mandatory runtime/service registry/executor was added;
- native ownership remains correct;
- no panic can unwind across foreign ABI;
- no unnecessary allocation/copy/lock/runtime layer was added;
- simulator/device/platform builds pass where relevant;
- `--no-default-features` checks pass for core crates once available;
- formatting/lints/tests pass;
- performance-sensitive changes have evidence;
- documentation is updated when invariants or ABI assumptions change.
