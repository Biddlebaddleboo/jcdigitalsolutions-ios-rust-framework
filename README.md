# JC Digital Solutions iOS Rust Framework

A reusable Rust-first native iOS framework focused on near-zero incremental runtime overhead, direct use of Apple's public APIs, and App Store-compatible deployment.

## Core idea

Applications should be able to keep nearly all application logic in Rust while using native UIKit/Foundation/Core* objects directly. The framework is not a virtual DOM, not a custom renderer, not a JavaScript bridge, and not a Swift wrapper around a Rust core.

The intended call path for ordinary Objective-C APIs is:

```text
Rust application
  -> thin Rust helper when useful
  -> objc2
  -> Objective-C ABI / objc_msgSend
  -> UIKit / Foundation / other Apple framework
```

For public C APIs the path is direct Rust FFI.

## Hard guarantees

- The framework repository contains no Swift source files.
- Shipping features use only public, App-Store-compatible Apple interfaces.
- `objc2` is the default Objective-C interoperability layer.
- Framework abstractions must remain thin and must justify any allocation, copy, dynamic dispatch, locking, or duplicated state.
- UIKit objects remain UIKit objects.
- Ordinary framework/application computation should use normal optimized Rust data structures rather than Objective-C objects.
- Swift-only public APIs may eventually be reached through narrowly scoped Swift-ABI interoperability implemented in Rust.
- An application may temporarily own a microscopic Swift shim for an unsupported API, but this repository will not.

## Performance objective

At Apple API boundaries, Rust should approach equivalent Objective-C/C overhead as closely as practical.

Outside the Apple object boundary, code should target ordinary optimized Rust/C++ performance and should not inherit Objective-C object/message overhead unnecessarily.

Performance claims must be measured against native baselines.

## Project status

The repository is currently in architecture/planning bootstrap. Read `AGENTS.md` before making any implementation change. Temporary `PLAN*.md` files will define execution scope once the first implementation plan is approved.

## Documentation

- `docs/ARCHITECTURE.md` — architecture and boundaries.
- `docs/PERFORMANCE.md` — performance model and benchmark policy.
- `docs/OWNERSHIP.md` — Rust/Objective-C lifetime rules.
- `docs/UNSAFE.md` — unsafe/ABI policy.
- `docs/APP_STORE_COMPLIANCE.md` — public-API and review requirements.
- `docs/IOS_BUILD.md` — build/signing/packaging direction.
- `docs/OBJC_INTEROP.md` — objc2, delegates, Blocks, callbacks.
- `docs/SWIFT_ABI.md` — zero-Swift-source policy and future Swift-ABI work.
- `docs/API_DESIGN.md` — when an abstraction should or should not exist.
