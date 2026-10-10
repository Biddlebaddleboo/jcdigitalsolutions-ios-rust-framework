# PLAN_CAPABILITIES_BACKGROUND_EXECUTION.md — Workstream D23: Background-Execution Lease

## Status

D23 implements a portable static backend contract for one finite native background-execution lease, an explicit end operation, and a cooperative expiry signal. It is independent from D20 app-refresh scheduling.

## Objective

Expose a small no-std contract that lets an app begin one backend-owned lease, check whether its native expiry signal has fired, and explicitly end the lease.

## Dependencies

- A core error category through `framework-core`
- B28 `ios-background-execution` as one UIKit backend
- No allocator, executor, scheduler, task registry, or additional portable dependency

## Write scope

- `PLAN_CAPABILITIES_BACKGROUND_EXECUTION.md`
- `crates/framework-background-execution/**`
- `docs/capabilities/background-execution.md`

Root owns workspace and lock integration, capability JSON and counts, shared indexes, aggregate plans, CI, and aggregate validation docs. Do not edit those paths.

## Contract

- `BackgroundExecution<B>` statically forwards to one selected `BackgroundExecutionBackend`; it adds no allocation or runtime backend lookup.
- `BackgroundExecutionBackend::begin` receives the backend's context and returns one backend-owned lease or a portable `framework-core` error.
- A `BackgroundExecutionLease` exposes an `ExpirySignal` and an explicit consuming `end` operation.
- The app should end the lease as soon as its bounded work is complete; a backend may also end it from a native expiry callback or a drop fallback.
- Expiry is cooperative: it does not interrupt or cancel Rust work. Callers check between bounded work units and stop promptly when expiry is true.
- A successful begin provides no duration, extra runtime guarantee, scheduler launch, continued execution, or work-completion promise.
- No run request, recurrence, history, durable task state, cancellation API, task identifier, or global registry exists in D23.

## Boundaries

- No `BGTaskScheduler`, `BGAppRefreshTask`, background processing task, BackgroundAssets, URLSession transfer, notification, push, or app-delegate API.
- No `std`, `alloc`, Objective-C, UIKit, Swift, or platform-native value in the portable contract.
- No C ABI or async executor.

## Checks

- `cargo +1.94.1 check --locked -p framework-background-execution --no-default-features`
- `cargo +1.94.1 clippy --locked -p framework-background-execution --all-targets -- -D warnings`
- `cargo +1.94.1 fmt --manifest-path crates/framework-background-execution/Cargo.toml -- --check`

## Local validation evidence

Rust 1.94.1 checks passed in the isolated D23 worktree based on `8b26a426d0e92d97ba2d0456346218dce8e1e8f1` on 2026-10-09:

- `cargo +1.94.1 check --locked --offline -p framework-background-execution --no-default-features`
- `cargo +1.94.1 clippy --locked --offline -p framework-background-execution --all-targets -- -D warnings`
- `cargo +1.94.1 fmt --manifest-path crates/framework-background-execution/Cargo.toml -- --check`
- `git diff --check`

The portable guide example now calls `BackgroundExecutionLease::expiry()`; `expiry_signal()` is an iOS adapter convenience, not part of the D23 portable trait. The crate has no dedicated test files or inline test module, and no test command was run. B28/G22 files remain read-only; no UIKit runtime behavior is claimed.

## Handoff

Report public names, dependency set, exact checks, expiry/end semantics, changes, and unresolved assumptions.
