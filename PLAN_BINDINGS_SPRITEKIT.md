# PLAN_BINDINGS_SPRITEKIT.md — Workstream F11: iOS SpriteKit Position C ABI

## Objective

Expose D64/B70's detached SpriteKit node-position slice through one opt-in C feature. Keep
`NativeSkNode`, Objective-C types, and Rust ownership wrappers behind an opaque pointer handle.

## Stable B70 contract

- `framework-spritekit::SpriteNodePosition` accepts finite `f64` x/y values only.
- `ios-spritekit::IosSpriteNode` creates one detached `SKNode`, reads its parent-local position,
  and replaces that position; it creates no scene, view, hierarchy, or renderer.
- B70 requires a `MainThread` proof at create, rechecks the main thread at get/set, and stores an
  explicitly `!Send`/`!Sync` retained node wrapper.
- B70's API floor is iOS 7.0. Its SDK-specific link validation floors are device 12.0 and
  Simulator 14.0; these are not a changed API floor.

## C contract

- Cargo feature `ios-spritekit` is off by default and enables only `framework-spritekit`,
  `ios-spritekit`, and `ios-runtime` for the iOS target.
- Header: `framework_ios_spritekit.h`; opaque handle: `FrameworkIosSpriteKitNode`.
- Exports: `framework_ios_spritekit_node_create`,
  `framework_ios_spritekit_node_get_position`,
  `framework_ios_spritekit_node_set_position`, and
  `framework_ios_spritekit_node_destroy`.
- Create initializes the output handle to null. Get initializes each non-null output coordinate to
  zero. Null required outputs and non-finite coordinates return `FRAMEWORK_STATUS_INVALID_ARGUMENT`.
- All iOS entry points require the main thread. Create checks before SpriteKit calls; get/set check
  before handle dereference. Off-main access returns `FRAMEWORK_STATUS_UNAVAILABLE`.
- Destroy returns `FRAMEWORK_STATUS_UNAVAILABLE` off-main before reading or changing the handle
  slot, allowing the host to retry on main. On main, a null slot or null handle returns
  `FRAMEWORK_STATUS_OK`; a live handle slot is cleared before its unique allocation is dropped.
- Each successful create has one unique opaque handle. Calls on it must not be concurrent or race
  destroy. Do not copy or destroy aliases. No native object pointer crosses the header.
- Valid non-iOS operations return `FRAMEWORK_STATUS_UNSUPPORTED`; create and get outputs remain
  initialized. No mock handle exists.
- Status exports contain Rust panics as `FRAMEWORK_STATUS_PANIC`; backend errors use
  `FrameworkStatus::from_error`.

## Scope

- `bindings/c/Cargo.toml`
- `bindings/c/src/lib.rs` and `bindings/c/src/ios_spritekit.rs`
- `bindings/c/include/framework_ios_spritekit.h`
- `bindings/c/abi-manifest.json`
- `bindings/c/check-ios-spritekit.sh`
- `docs/bindings/ios-spritekit.md`
- `PLAN_BINDINGS_SPRITEKIT.md`

Root owns `Cargo.lock`, `PLAN_BINDINGS.md`, CI, shared documentation indexes, and capability
counts. Do not edit those files in F11.

## Validation gate

The named gate must not add or run tests or execute consumer/probe binaries. It must check manifest
JSON and shell syntax, format the C ABI crate, compare F11 header symbols to the ABI manifest, verify the feature
is absent from the default and non-iOS dependency graphs, and verify the iOS feature graph includes
only the three declared capability dependencies. It must run locked host/device/Simulator check and
strict Clippy gates, build host and arm64 Apple release archives, compile and link C11/C++17
consumers without execution, inspect F11 symbols and exact framework imports, and check deployment
metadata. Device and Simulator probes use 12.0 and 14.0 respectively; record those separately from
the iOS 7.0 API floor.

## Status and validation record

F11 source, header, ABI manifest, guide, and check script are integrated. On Rust 1.94.1, Xcode
26.6 build 17F113, and iOS SDK 26.5, `sh bindings/c/check-ios-spritekit.sh` passed. It validates
host/device/Simulator locked checks and strict Clippy, manifest/header symbol parity, feature
isolation, host/device/Simulator release archives, and C11/C++17 host/device/Simulator compile and
link probes. The host archive exports exactly the four F11 symbols and imports no SpriteKit or
Objective-C symbols. Device and Simulator C imports are CoreFoundation, Foundation, SpriteKit,
UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; C++ adds `libc++.1.dylib`. Linked device and
Simulator probes report `minos` 12.0 and 14.0. These are SDK-specific validation targets; the B70
API floor remains iOS 7.0. The probes were linked and inspected, not executed; no tests were run.

- `python3 -m json.tool bindings/c/abi-manifest.json > /dev/null`
- `sh -n bindings/c/check-ios-spritekit.sh`
- `cargo fmt --manifest-path bindings/c/Cargo.toml --package framework-c-api -- --check`
- `git diff --check`
- `sh bindings/c/check-ios-spritekit.sh`

The root-owned `Cargo.lock` was refreshed by the orchestrator before the locked F11 gate. F11 did
not edit it.

Do not claim scene integration, rendering, hierarchy, animation, physics, native-handle access,
cross-platform SpriteKit, parity, or performance.
