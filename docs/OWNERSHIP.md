# Ownership and Lifetimes

## Principle

Do not invent a new ownership model for Apple objects.

Use Rust ownership for Rust data and Objective-C ownership for Objective-C objects, with the smallest correct interoperability layer between them.

## Objective-C objects

Preferred objc2 ownership types:

- `Retained<T>` — owned strong reference;
- borrowed `&T` — non-owning reference with an established lifetime;
- `Weak<T>` — Objective-C weak semantics;
- `Allocated<T>` — allocated but not yet fully initialized object where the API requires it.

A `Retained<T>` already corresponds to native retain/release behavior. Do not wrap it in `Rc` or `Arc` by default.

## UIKit hierarchy

UIKit owns the native view/controller hierarchy according to UIKit semantics.

Rust wrappers must not mirror that hierarchy into a second ownership graph.

## Rust application state

Prefer ordinary unique ownership.

For main-thread UI state, avoid `Arc<Mutex<_>>` unless actual cross-thread shared mutation is required.

Stable heap allocation may be used when Objective-C callbacks need stable context addresses, but lifetime and reentrancy must be explicit.

## Borrowing native values

Borrow native strings/data/views where safe instead of cloning/retaining them automatically.

A convenience API must not silently turn every borrowed Apple value into an owned Rust allocation.

## Retain cycles

Audit:

- Blocks capturing owners;
- delegates retained by owners;
- target objects;
- closures retaining view controllers;
- callbacks retaining registries that retain callbacks.

Use weak references or explicit teardown where required.

## Autorelease

Respect autorelease conventions.

Worker threads or tight loops creating Objective-C temporaries may need explicit autorelease-pool management.

Do not add pools to hot paths blindly; benchmark and follow platform semantics.

## Reentrancy

UIKit can synchronously call back into application code during an operation.

Do not expose permanent mutable global references that can alias under reentrancy.

Prefer split state, narrowly scoped mutable borrows, callback IDs, or queued work when nested access would violate Rust aliasing.

## FFI panic rule

Rust panics must not unwind through Objective-C or C ABI frames.

Foreign entrypoints must contain panic behavior explicitly. Evaluate `panic=abort` for final framework profiles, but do not rely on it as a substitute for correct boundary design.
