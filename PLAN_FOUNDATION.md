# PLAN_FOUNDATION.md — Workstream A: Portable Foundation

## Objective

Create the V1 Cargo workspace and the shared portable Rust foundations that every other workstream consumes.

This workstream is the single owner of shared semantic primitives. It must land before other workstreams modify shared interfaces.

## Status and evidence

All five foundation crates have `#![no_std]` roots; no third-party crate appears in their dependency trees

- `framework-core` uses only `core`
- `framework-alloc`, `framework-async`, `framework-abi`, and `framework-platform` each depend only on `framework-core`
- `framework-alloc` exposes reusable `GenerationalSlab` and `BitSet` values, but no production crate uses them and no benchmark proves a runtime need
- `framework-abi` exposes `framework_owned_buffer_copy`, which copies a `FrameworkSlice` into a framework-owned buffer with fixed-width length checks, fallible allocation, empty-on-failure output, and explicit destruction through `framework_owned_buffer_destroy`
- `FrameworkErrorHandle` remains a fixed-width scalar; the opt-in C ABI separately creates directly owned error-detail objects through `FrameworkErrorDetailHandle`

Host result: Rust `1.94.1` no-default checks for these five crates passed again on 2026-10-09 as part of the 47-crate `cargo xtask no-std-check` run. Tests, strict Clippy, rustdoc, and codegen status passed on 2026-10-08. On 2026-10-10, the owned-buffer factory passed no-default Cargo checks for `framework-abi` and `framework-c-api`, the optimized C static-library build and archive symbol inspection, formatting, manifest parsing, C example syntax, and static diff checks; no tests or runtime checks were run. No host check proves device or simulator linkage, runtime behavior, 32-bit target execution, or a production need for `framework-alloc`

F35 error-detail evidence on 2026-10-10: `framework-abi --no-default-features` and `framework-c-api` checks passed; the release C archive built and its exported symbol set matched the ABI manifest; Rust formatting, manifest inspection, C11/C++17 header syntax, and the C example syntax passed. No tests, linked example, or runtime checks were run; no device/simulator or 32-bit proof is claimed.

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
- fixed-width semantic value layouts and `CompactHandle` encode/decode without pointer IDs;
- `BitSet` word edges, set/get, clear, count, empty domain, and out-of-range rejection; V1 has no packed-value API;
- valid and invalid `OperationState` transitions, one-shot success/failure/cancellation, and repeated calls;
- zero-sentinel checks for `CapabilityId`, `OperationId`, `Generation`, `PlatformErrorCode`, `CompactHandle`, `FrameworkOperationHandle`, and `FrameworkErrorHandle`;
- handle generation/stale-handle behavior;
- wraparound policy;
- cancellation/completion race model using controlled synchronization;
- C ABI layout/size assertions, fixed status codes and category mapping, zero-handle sentinels, `FrameworkOptionsV1` header values, and `framework_owned_buffer_copy` null output, null/non-null empty input, overflow, allocation failure, copy, and destruction cases;
- `FrameworkOwnedBuffer::try_from_vec` ownership transfer and one call to the C destructor on the original descriptor;
- `catch_unwind_status` under the optional `std` feature.

No real sleeps

## Validation commands

At minimum:

```bash
cargo +1.94.1 check --locked -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --no-default-features
cargo +1.94.1 test --locked -p framework-core
cargo +1.94.1 test --locked -p framework-alloc
cargo +1.94.1 test --locked -p framework-async
cargo +1.94.1 test --locked -p framework-abi --all-features
cargo +1.94.1 test --locked -p framework-platform
cargo +1.94.1 clippy --locked -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --all-targets -- -D warnings
cargo +1.94.1 clippy --locked -p framework-abi --all-features --all-targets -- -D warnings
cargo +1.94.1 run --locked -p xtask -- codegen-audit
```

The `codegen-audit` inspects optimized LLVM IR for four synthetic `OperationId` probes on the host and installed iOS targets; it is structural evidence for fixed-width accessors, not a benchmark, full-program LTO check, link check, or runtime claim

### Host check record (2026-10-08)

- `cargo +1.94.1 check --locked -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --no-default-features` — PASS
- `cargo +1.94.1 test --locked -p framework-core` — PASS, 7 tests
- `cargo +1.94.1 test --locked -p framework-alloc` — PASS, 5 tests
- `cargo +1.94.1 test --locked -p framework-async` — PASS, 7 tests incl controlled cancellation/completion race
- `cargo +1.94.1 test --locked -p framework-abi --all-features` — PASS, 7 tests incl `catch_unwind_status`
- `cargo +1.94.1 test --locked -p framework-platform` — PASS, 2 tests
- `cargo +1.94.1 clippy --locked -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --all-targets -- -D warnings` — PASS
- `cargo +1.94.1 clippy --locked -p framework-abi --all-features --all-targets -- -D warnings` — PASS
- `cargo +1.94.1 doc --locked --all-features -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --no-deps` — PASS
- `cargo +1.94.1 run --locked -p xtask -- codegen-audit --output target/xtask/codegen-audit-foundation-20261008.json` — PASS for `x86_64-apple-darwin`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`, and `x86_64-apple-ios`
- `cargo +1.94.1 tree --locked -p framework-core -p framework-alloc -p framework-async -p framework-abi -p framework-platform --prefix none` — `framework-core` has no dependency; each other foundation crate has only the `framework-core` edge
- `rustfmt +1.94.1 --check crates/framework-core/src/lib.rs crates/framework-async/src/lib.rs crates/framework-alloc/src/lib.rs crates/framework-abi/src/lib.rs` — PASS
- `git diff --check` — PASS

The codegen report is structural IR evidence for `OperationId` only; it makes no runtime cost claim. The host has no 32-bit target, and no device or simulator app ran. Prior audit files remain in `target/xtask/codegen-audit/build.pre-foundation-audit-20261008` and `target/xtask/codegen-audit/fixture.pre-foundation-audit-20261008`

## Recorded evidence and limits

- The representation table in [docs/core/FOUNDATION.md](docs/core/FOUNDATION.md) records size, alignment, array stride, scalar domains, sentinel values, slab capacity, and generation wrap behavior
- [docs/core/ASYNC_AND_OWNERSHIP.md](docs/core/ASYNC_AND_OWNERSHIP.md) records atomic phase values, compare-exchange ordering, result publication, waker-slot ordering, future drop behavior, and backend callback lifetime
- [docs/core/C_ABI.md](docs/core/C_ABI.md) records C layouts, pointer-width-dependent fields, owned-buffer rules, status values, callback lifetime, options headers, and panic limits
- `framework-alloc` remains optional reusable infrastructure without a production caller or benchmark evidence; a production caller or benchmark must justify any required runtime use
- `framework_owned_buffer_copy` now creates a C-owned descriptor from caller-provided bytes; no tests were added or run for this change
- `FrameworkErrorDetailHandle` is a direct opaque pointer with explicit create/view/destroy semantics; it does not encode pointers into the existing `FrameworkErrorHandle(u64)`
- Other workstreams must keep fixed-width semantic IDs, zero sentinels, `CompactHandle` field order, generation wrap from `u32::MAX` to `1`, the single cancellation/completion winner, operation-state lifetime through backend callback teardown, and one destroy call on the original owned-buffer descriptor
- The host test and codegen gates do not prove iOS device or simulator linking, platform runtime behavior, or 32-bit target execution

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
- exact test, Clippy, and codegen commands/results;
- any semantics that later workstreams must not change.
