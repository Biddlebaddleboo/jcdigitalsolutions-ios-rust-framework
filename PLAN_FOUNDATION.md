# PLAN_FOUNDATION.md — Workstream A: Portable Foundation

## Objective

Create the V1 Cargo workspace and the shared portable Rust foundations that every other workstream consumes.

This workstream is the single owner of shared semantic primitives. It must land before other workstreams modify shared interfaces.

## Implementation scope

Inspect first:
- `AGENTS.md`
- `docs/ARCHITECTURE.md`
- `docs/API_DESIGN.md`
- `docs/PORTABILITY_AND_ABI.md`
- `docs/OWNERSHIP.md`
- `docs/UNSAFE.md`
- `docs/DOCUMENTATION.md`
- `docs/TESTING_AND_PARITY.md`

Proposed write scope:
- `Cargo.toml`
- `rust-toolchain.toml`
- `.gitignore`
- workspace lint/profile configuration
- `crates/framework-core/**`
- `crates/framework-alloc/**`
- `crates/framework-async/**`
- `crates/framework-abi/**`
- `crates/framework-platform/**`
- shared foundational docs introduced under `docs/core/**`

Read-only dependencies:
- existing research docs
- no platform backend code yet

## Verified facts

- Portable core must start as genuine `#![no_std]`.
- `alloc` is allowed only where required.
- `std` types must not leak into portable contracts.
- Rust `Future` may be exposed without requiring a global executor.
- Future 32-bit offset/compressed-pointer support must remain possible.
- Internal data may be compact/bit-packed; public API remains semantic.
- Stable C ABI primitives must not expose Rust layout.
- Significant dependencies must remain replaceable and minimized.

## Proposed crate responsibilities

### `framework-core`
`#![no_std]`, preferably allocator-free.

Proposed symbols:
- `Platform`
- `Capability`
- `CapabilityId`
- `Availability`
- `ErrorKind`
- `PlatformErrorCode`
- `Error`
- `Result<T>`
- `PermissionState`
- `AuthorizationState`
- `OperationId`
- `Generation`
- `CompactHandle`
- `Duration`/`Instant`-like framework primitives only if required and without pretending wall-clock/system time is portable

Rules:
- fixed-width semantic fields;
- no public `usize` IDs/counters except address-space-sized quantities;
- error core must not require allocation;
- optional extended diagnostic detail may live in alloc-enabled layer.

### `framework-alloc`
`#![no_std]` + `alloc`.

Proposed internal types:
- `PackedFlags<T>` or capability-specific typed status newtypes;
- `GenerationalIndex` with documented index/generation widths;
- `Slab<T>` / arena primitives only if measurements and use cases justify them;
- owned byte/string helpers only where `alloc` is required;
- hot/cold state split helpers if they reduce duplication across capabilities.

Do not recreate `Vec`, `String`, hashing, or collections without a concrete reason.

### `framework-async`
`#![no_std]` and avoid `alloc` where possible.

Proposed symbols:
- `CancellationToken`
- `CancellationRegistration`
- `OperationState<T, E>`
- `Completion<T, E>`
- `OperationFuture<'a, T, E>` or equivalent runtime-neutral future adapter
- exactly-once completion primitive

Required state semantics:
- created -> running -> completed
- created/running -> cancellation requested
- cancellation races with completion resolve exactly once
- drop of caller-facing future does not automatically cause use-after-free
- backend teardown and callback lifetime are explicit
- no implicit thread/task spawning

Prefer atomics/compact state words where needed, with explicit memory-order rationale.

### `framework-abi`
`#![no_std]` + `alloc` only for explicitly owned cross-ABI buffers.

Proposed C ABI value forms:
- `FrameworkStatus`
- `FrameworkSlice`
- `FrameworkStr`
- `FrameworkOwnedBuffer`
- `FrameworkOperationHandle`
- `FrameworkErrorHandle`
- callback function pointer typedefs
- versioned option structs

Requirements:
- `#[repr(C)]` only where externally specified;
- explicit integer representations for enums/status;
- explicit destructor functions;
- pointer+length strings/bytes;
- no panic unwind;
- no Rust enum/layout/reference/trait object exposure;
- every owned object has an unambiguous creator/destroyer.

### `framework-platform`
Compile-time selection helpers only:
- target platform marker types;
- sealed/internal backend association traits if needed;
- no runtime registry or service locator.

## Workspace configuration

Root workspace should:
- use globs so later workstreams can add crates without repeated root member edits;
- centralize dependency versions/features;
- enable strict lints;
- deny unsafe operation in unsafe fn where practical;
- configure release/LTO settings appropriate for benchmark builds without forcing one app profile on all consumers;
- avoid a workspace-wide default dependency that introduces `std`.

Do not add `cargo-deny`, `criterion`, proptest, bindgen, cbindgen, serde, tokio, tracing, anyhow, thiserror, or other convenience dependencies automatically. Add only after specific justification.

## Compact representation requirements

For every repeated foundational type record:
- `size_of`;
- `align_of`;
- stride;
- bit allocation if packed;
- capacity;
- invalid/sentinel states;
- generation wrap behavior;
- atomicity/memory ordering if concurrent.

Proposed target principles, not hard byte numbers:
- handles should normally be `u32`/`u64` semantic values rather than native pointers;
- small permission/state enums should use explicit narrow repr where externally relevant;
- common error headers should keep verbose strings/platform objects out of hot representation.

## Future compact-pointer compatibility

V1 must not implement pointer compression.

Foundation must make later implementation possible by:
- using fixed-width semantic handles/indexes;
- isolating actual pointers in unsafe/platform types;
- avoiding serialized raw pointers;
- avoiding public pointer-derived IDs;
- keeping arenas/handle mapping replaceable.

Add tests that compile/validate fixed-width assumptions independent of `usize`.

## Error handling

Errors must preserve:
- stable portable category;
- optional platform code;
- optional cold diagnostic/platform detail in higher layers.

Do not allocate a diagnostic string on every success/failure path unless required.

## Tests

Deterministic unit/property tests:
- packed encode/decode;
- all valid/invalid state transitions;
- handle generation/stale-handle behavior;
- wraparound policy;
- cancellation/completion race model using controlled synchronization;
- C ABI layout/size assertions;
- FFI create/destroy ownership;
- panic-containment wrappers.

No real sleeps.

## Validation commands

At minimum:

```bash
cargo check -p framework-core --no-default-features
cargo check -p framework-alloc --no-default-features
cargo check -p framework-async --no-default-features
cargo check -p framework-abi --no-default-features
cargo test -p framework-core
cargo test -p framework-alloc
cargo test -p framework-async
cargo test -p framework-abi
cargo clippy -p framework-core -p framework-alloc -p framework-async -p framework-abi --all-targets -- -D warnings
```

Also inspect optimized code for trivial wrappers to verify zero-cost static abstractions.

## Non-goals

- no iOS APIs;
- no Swift ABI;
- no Python;
- no full allocator replacement;
- no custom crypto;
- no universal executor;
- no runtime plugin system;
- no pointer compression in V1.

## Handoff

Report:
- final crate graph;
- shared public symbols;
- size/alignment table;
- dependencies added and rationale;
- test commands/results;
- any semantics that later workstreams must not change.