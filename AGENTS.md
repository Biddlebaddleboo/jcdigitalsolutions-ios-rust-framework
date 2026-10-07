# AGENTS.md

## Purpose

This repository builds a reusable, Rust-first native iOS framework whose normal application code can be written almost entirely in Rust while still using Apple's native frameworks directly.

The framework is intentionally not a Flutter/React-Native-style runtime, not a virtual DOM, not a custom renderer, and not a Swift wrapper around a Rust core. UIKit and other Apple objects remain native Apple objects. Rust should add as close to zero incremental runtime cost as practical.

These rules are architectural invariants. Do not weaken them for convenience.

## Hard rules

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

### Rust is the primary implementation language

Use Rust by default.

C, C++, Objective-C, or Objective-C++ may be used only when a concrete ABI, toolchain, or platform requirement makes that the smallest correct solution. Keep such code microscopic and document why Rust alone is insufficient.

### objc2 is the default Objective-C interoperability layer

Use `objc2` and its framework crates for public Objective-C APIs unless there is a measured or correctness-based reason not to.

Do not reimplement `objc_msgSend`, Objective-C ownership machinery, Blocks, or framework bindings simply because lower-level code looks faster.

Direct Objective-C ABI calls or narrowly scoped assembly are allowed only when at least one of the following is true:
1. the required ABI surface is not correctly expressible through current bindings;
2. generated-code inspection shows avoidable incremental overhead;
3. a benchmark demonstrates a meaningful improvement.

Any bypass must preserve Objective-C ownership, calling conventions, error semantics, and App Store compliance.

### Optimize the Rust side as Rust, not as Objective-C

Only objects that must participate in Apple's object model should be Objective-C objects.

Keep application/framework-owned computation in ordinary Rust representations when possible:
- structs and enums;
- slices and `Vec`;
- compact/generational IDs;
- static dispatch and generics;
- arenas/slabs when justified;
- cache-friendly layouts;
- SIMD or architecture-specific code when measured.

Do not turn ordinary application state into `NSObject` subclasses.

### No unnecessary runtime layer

Do not introduce, unless a concrete requirement proves necessary:
- a garbage collector;
- a framework-wide `Arc`/`Rc` ownership model;
- a global UI-state mutex;
- JSON/serialization between Rust and Apple APIs;
- IPC between framework layers;
- a virtual UI tree;
- a reconciliation engine;
- a custom renderer for ordinary UIKit views;
- string-based method routing;
- reflection for ordinary property/method access;
- boxed dynamic callbacks for every event;
- a mandatory async runtime such as Tokio;
- duplicate native object models.

### Abstraction must justify itself

Default rule: do not wrap an `objc2` or Apple API merely to hide it.

A framework abstraction should exist only when it:
- removes recurring ABI/lifetime boilerplate;
- enforces an important invariant;
- materially improves Rust ergonomics;
- exposes a genuinely portable concept;
- enables a measurable optimization.

Prefer thin, inlineable wrappers. Preserve access to the underlying native object so uncommon Apple APIs do not have to wait for framework wrapper coverage.

### UIKit first

V1 is UIKit-native.

Do not implement a SwiftUI clone, virtual DOM, custom text engine, custom scrolling system, custom accessibility tree, or custom renderer in the first architecture.

A future Metal/custom-rendering backend is a separate decision and must not distort the UIKit-native core.

## Ownership and lifetime rules

Use Objective-C ownership semantics directly:
- `Retained<T>` for owned strong Objective-C references;
- borrowed references where ownership is unnecessary;
- `Weak<T>` where weak semantics are required;
- `Allocated<T>` during initialization where appropriate.

Do not wrap `Retained<T>` in `Arc` or `Rc` by default. `Retained<T>` already represents Apple's native retain/release ownership.

UIKit owns its native view/controller hierarchy. Rust owns Rust application state and framework semantics.

Avoid unnecessary retain/release traffic. Treat native ownership operations as Apple costs only when they are required by correct semantics; extra framework-generated retains/releases are optimization bugs.

## Main-thread rules

Prefer encoding UI-thread correctness with `objc2::MainThreadMarker` or an equally zero-cost typed capability.

Do not replace compile-time/main-thread capability checking with repeated runtime boolean checks unless unavoidable.

UI state should generally be main-thread owned. Do not default to `Arc<Mutex<...>>` for UI state.

Handle UIKit reentrancy explicitly. Never fabricate permanent mutable references to global application state that can alias during synchronous callbacks.

## Objective-C classes, delegates, and target/action

Use Rust-defined Objective-C classes through `objc2::define_class!` where appropriate.

Create Objective-C bridge objects only where Apple APIs require an Objective-C object, for example:
- application delegate;
- control target;
- table/list data source or delegate;
- text-input delegate;
- notification delegate;
- navigation delegate.

Keep bridge objects narrow.

For common callbacks, prefer compact IDs, statically known functions, or similarly cheap dispatch over `Arc<Mutex<Box<dyn FnMut(...)>>>`.

Handle stale callback IDs, teardown, reentrancy, deallocation, and retain cycles.

A Rust panic must never unwind through an Objective-C/C ABI boundary.

## Blocks

Use `block2` for Apple Blocks.

Distinguish escaping from nonescaping callbacks. Avoid heap allocation or retained closure state when the Apple API and Rust callback shape do not require it.

Audit block capture graphs for retain cycles.

## Strings and bytes

Do not eagerly convert every `NSString` into `String`, or every `NSData` into `Vec<u8>`.

Distinguish:
- borrowed native strings;
- retained native strings;
- temporary strings;
- static strings;
- owned Rust strings;
- borrowed native bytes;
- retained `NSData`;
- borrowed byte slices;
- owned `Vec<u8>`.

Measure conversion and copy costs before adding convenience conversions to hot APIs.

## Swift-only public APIs

The framework contains no Swift source.

Escalation order:
1. public Objective-C interface via `objc2`;
2. public C/CoreFoundation/Darwin interface via Rust FFI;
3. add/generate a missing public binding;
4. implement equivalent convenience behavior over documented lower-level public APIs;
5. for a genuinely public Swift-only API, implement the minimum required Swift ABI interoperability in Rust;
6. leave the API unsupported until a robust compliant path exists.

Do not build a general Swift compiler/runtime clone speculatively.

Any Rust Swift-ABI implementation must be driven by a concrete public Apple API and document the exact ABI assumptions it relies on.

## Unsafe Rust

Unsafe is acceptable and expected in a native platform framework.

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

Unsafe is not a performance feature by itself. Use it when it removes a proven abstraction constraint or implements required native semantics.

## Assembly

ARM64 assembly is allowed for:
- ABI thunks;
- measured hot paths;
- operations where LLVM output is demonstrably inferior.

Before adding assembly:
1. inspect optimized compiler output;
2. benchmark the existing implementation;
3. prove the assembly changes the relevant cost;
4. document ABI/clobber assumptions;
5. provide a safe/correct fallback where needed.

Do not use assembly merely to appear lower level.

## Performance standard

There are two targets.

### Apple boundary

For an Objective-C Apple API, Rust + `objc2` should approach the cost of equivalent Objective-C as closely as practical.

If an Objective-C call ultimately requires `objc_msgSend`, that dispatch is an Apple/Objective-C cost, not a Rust penalty.

### Framework/application computation

Code not required to use Apple's object model should target ordinary optimized Rust/C++ performance and may outperform conventional object-heavy Objective-C implementations.

Measure:
- launch time;
- RSS;
- allocations;
- retain/release traffic;
- callback dispatch;
- UI property mutation;
- navigation;
- list/data-source behavior;
- string/data conversion;
- networking setup/completion/body copies;
- binary size where relevant.

Performance claims must be backed by reproducible measurements.

## Benchmark comparison

For apples-to-apples UIKit work, compare at least:
- Objective-C/UIKit;
- Swift/UIKit;
- Rust/UIKit through this framework.

SwiftUI may be included as an architectural comparison, but do not treat it as the zero-overhead UIKit baseline.

Inspect optimized assembly for important ABI hot paths.

Do not optimize benchmarks by weakening functionality.

## API design

The framework should make common iOS development much less cumbersome than raw `objc2` without creating a second UI framework.

Good abstraction:
- hides repetitive selector/delegate/block/lifetime boilerplate;
- stays thin;
- exposes native access;
- adds no hidden allocation/copy/lock in the hot path.

Bad abstraction:
- mirrors UIKit into a new framework object tree;
- requires runtime reconciliation;
- boxes every operation;
- serializes values between layers;
- prevents direct access to native capabilities.

## Networking and async

Prefer Apple's native networking stack.

Do not require Tokio or another general-purpose executor.

Start with direct callback/Block/delegate semantics. Add `Future` adapters only when they remain cheap and useful.

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
- only public Apple APIs are used;
- no private entitlement/API shortcut was introduced;
- native ownership remains correct;
- no panic can unwind across foreign ABI;
- no unnecessary allocation/copy/lock/runtime layer was added;
- simulator and relevant device builds pass;
- formatting/lints/tests pass;
- performance-sensitive changes have evidence;
- documentation is updated when invariants or ABI assumptions change.
