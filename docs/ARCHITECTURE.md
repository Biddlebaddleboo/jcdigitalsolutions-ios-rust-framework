# Architecture

## Goal

Provide a native iOS development foundation where Rust is the application language, Apple frameworks remain native, and the framework contributes as little runtime machinery as possible.

## Boundary model

The architecture has two domains.

### Rust domain

Keep these as ordinary Rust whenever they do not need Apple's object model:

- application state;
- business logic;
- collections and models;
- algorithms;
- parsing;
- protocol logic;
- state machines;
- scheduling decisions;
- non-Apple-specific utilities.

Prefer static dispatch, plain structs/enums, compact IDs, contiguous storage, and explicit ownership.

### Apple domain

Use native Apple objects only where required:

- UIKit views/controllers;
- delegates/protocol objects;
- Foundation types required by API contracts;
- Blocks;
- system framework handles;
- Objective-C runtime objects.

The framework should cross this boundary only when necessary.

## Default interoperability paths

### Objective-C APIs

```text
Rust
 -> optional thin helper
 -> objc2
 -> Objective-C ABI
 -> Apple framework
```

`objc2` is the default because it already provides typed Objective-C messaging, ownership integration, class/protocol support, and framework bindings without introducing a separate managed runtime.

### C APIs

Use direct Rust `extern "C"` bindings or established Rust bindings when they remain thin and correct.

### Swift-only public APIs

Do not add Swift source to this repository.

Investigate, in order:

1. public Objective-C exposure;
2. public C/CoreFoundation/Darwin exposure;
3. a missing Rust binding;
4. documented lower-level public primitives;
5. minimal Swift-ABI interoperability implemented in Rust.

If none is sufficiently robust, leave the feature unsupported until it can be implemented correctly.

## UI architecture

V1 is UIKit-native.

Do not create a parallel view tree. A `UILabel` is a `UILabel`, a `UIButton` is a `UIButton`, and a `UIViewController` remains a `UIViewController`.

Thin Rust helpers may reduce repetitive setup, lifetime, target/action, delegate, or error boilerplate, but the native object remains directly accessible.

Do not build:

- a virtual DOM;
- a reconciler;
- a SwiftUI clone;
- a framework renderer;
- a custom scrolling system;
- a custom text/layout engine;
- a duplicate accessibility tree.

## Threading

UIKit state is main-thread-owned by default.

Prefer a typed main-thread capability, such as `objc2::MainThreadMarker`, so correctness is encoded without repeated runtime checks.

Worker tasks may produce `Send` results and transfer only the required values back to the main thread.

Avoid global UI locks.

## Callback architecture

Use the minimum object machinery UIKit requires.

For target/action and delegate callbacks:

- create narrow Rust-defined Objective-C classes;
- prefer compact/generational callback IDs or static callback functions for common cases;
- avoid universal boxed closure storage;
- account for reentrancy;
- prevent stale callback invocation;
- prevent retain cycles;
- prevent unwinding across FFI.

## Ownership

Objective-C ownership should map directly to Apple retain/release semantics through objc2 ownership types.

Do not create a second reference-counting layer around every native object.

## Native escape access

Any ergonomic wrapper must allow direct access to the underlying native Apple object so uncommon APIs are not blocked by framework coverage.

## Future portability

A future Android or other-platform backend may share genuinely portable concepts, but iOS API design must not be distorted to force UIKit into an artificial lowest-common-denominator abstraction.
