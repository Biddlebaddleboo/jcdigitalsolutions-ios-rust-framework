# JC Digital Solutions Native Rust Framework

A reusable, high-level, platform-agnostic native application framework implemented primarily in Rust.

iOS is the first backend. The architecture is intentionally designed for future Android, macOS, Windows, Linux, and Web/WASM support with minimal application-code changes.

## Core idea

Applications should be able to keep almost all application logic in portable Rust while using native platform capabilities directly underneath.

The framework is not:
- a virtual DOM;
- a custom renderer/runtime;
- a JavaScript bridge;
- a Swift wrapper around a Rust core;
- a VM or managed universal object system.

The intended Rust call path is:

```text
high-level Rust application API
  -> portable Rust capability layer
  -> statically selected native backend
  -> native platform API
```

On iOS, native calls use direct C/CoreFoundation APIs, `objc2` for Objective-C APIs, and narrowly scoped Swift ABI support only for genuinely Swift-only public APIs.

## Language compatibility

Rust is the native fast path.

The same shared Rust core will also expose:
- a stable C ABI for C-compatible languages;
- optional C++ ergonomic wrappers;
- optional Python bindings;
- future language bindings where useful.

Rust callers are not routed through the C ABI.

Python remains optional and does not affect non-Python runtime/dependency cost.

## Portability model

The framework distinguishes:
1. portable capabilities;
2. portable capabilities with platform extensions;
3. platform-exclusive capabilities.

Portable contracts do not expose UIKit, JNI, Win32, DOM, or other platform-native types.

## Hard guarantees

- Rust-native path first; C ABI is for foreign-language compatibility.
- Portable code is designed `no_std`-first, aiming at `no_std + alloc` for most core/capability layers.
- Capabilities are fine-grained and independently importable.
- A small API must not pull large unrelated modules/frameworks.
- No mandatory framework runtime, global service registry, or universal async executor.
- Platform backends are selected statically where practical.
- The iOS backend contains no Swift source files.
- Shipping features use only public, supported platform APIs.
- `objc2` is the default Objective-C interoperability layer on Apple platforms.
- Framework abstractions must justify any allocation, copy, dynamic dispatch, locking, runtime lookup, or duplicated state.
- Platform-native escape hatches remain available.

## Performance objective

At native API boundaries, framework cost should approach the best equivalent native implementation as closely as practical.

For Rust, high-level portable wrappers should often optimize away entirely.

For C/C++, expected overhead should normally be limited to an ordinary C ABI call.

For Python, the goal is minimal framework-added overhead beyond CPython itself.

Performance claims must be measured.

## Project status

The repository is currently in architecture/planning bootstrap. Read `AGENTS.md` before making any implementation change. Temporary `PLAN*.md` files will define execution scope once the first implementation plan is approved.

## Documentation

- `docs/ARCHITECTURE.md` — architecture, portable/backend boundaries, and dependency direction.
- `docs/PORTABILITY_AND_ABI.md` — cross-platform, C ABI, Python, modularity, no_std, and binding rules.
- `docs/PERFORMANCE.md` — performance model and benchmark policy.
- `docs/OWNERSHIP.md` — Rust/Objective-C lifetime rules.
- `docs/UNSAFE.md` — unsafe/ABI policy.
- `docs/APP_STORE_COMPLIANCE.md` — public-API and review requirements.
- `docs/IOS_BUILD.md` — iOS build/signing/packaging direction.
- `docs/OBJC_INTEROP.md` — objc2, delegates, Blocks, callbacks.
- `docs/SWIFT_ABI.md` — zero-Swift-source policy and Swift-ABI work.
- `docs/API_DESIGN.md` — portable/high-level API design rules.
- `docs/research/` — Apple API and Swift ABI research corpus.
