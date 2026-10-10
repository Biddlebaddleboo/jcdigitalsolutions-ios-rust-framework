# PLAN_CAPABILITIES_BACKGROUND_TASKS.md — Workstream D20: App Refresh Tasks

## Status

D20 contract and D20-owned docs are complete. On 2026-10-09, Rust 1.94.1 with Xcode 26.6 (build 17F113) and iOS/iOS Simulator SDK 26.5 passed the following static gates:

- `cargo +1.94.1 check --locked -p framework-background --no-default-features`
- `cargo +1.94.1 clippy --locked -p framework-background --no-default-features --all-targets -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-background-tasks --target aarch64-apple-ios`
- `cargo +1.94.1 clippy --locked -p ios-background-tasks --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 check --locked -p ios-background-tasks --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked -p ios-background-tasks --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 fmt --manifest-path crates/framework-background/Cargo.toml -- --check` and `cargo +1.94.1 fmt --manifest-path platform/ios/ios-background-tasks/Cargo.toml -- --check`
- `RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked -p framework-background --no-default-features --no-deps`
- `RUSTDOCFLAGS="-D warnings" cargo +1.94.1 doc --locked -p ios-background-tasks --target aarch64-apple-ios --no-deps`
- `cargo +1.94.1 xtask docs-check`, `cargo +1.94.1 xtask zero-swift-source`, `sh platform/ios/ios-background-tasks/check-link-imports.sh`, and `git diff --check`

The link/import script passed both device and Simulator probe builds and its import, selected-symbol, exclusion, and deployment-floor assertions. No tests were added or run. Xcode 27.x qualification remains open. These static checks do not establish host launch setup, scheduler acceptance, run timing, callback delivery, expiry behavior, or runtime/device behavior

## Objective

Define one app-owned ID, one app-refresh request, a cooperative expiry signal, a result value, and a static backend contract

## Dependencies

- Workstream A core errors
- B25 `ios-background-tasks` as a separate adapter
- No executor, global registry, or new portable dependency

## Write scope

- `PLAN_CAPABILITIES_BACKGROUND_TASKS.md`
- `crates/framework-background/**`
- `docs/capabilities/background-tasks.md`

Root owns workspace and lock integration, the capability manifest and counts, shared indexes, aggregate plans, CI, and aggregate validation docs. Do not edit those paths

## Contract

- `AppRefreshTaskId::new` owns a non-empty string with no NUL byte; it keeps exact case and bytes with no trim or normalization
- The host app picks each ID and adds the exact string to its native app metadata
- `AppRefreshRequest` holds one ID and no date, interval, or run-count field
- For each accepted request, the OS may launch zero or one run; submit a new request for another run
- `AppRefreshBackend::submit_app_refresh` returns `Ok` only after backend acceptance; it does not promise a run or time
- A second pending request with the same ID replaces the prior pending request
- `cancel_app_refresh` asks to cancel a pending request only; it does not stop a run already in progress
- Register each ID once per app process before launch ends; a native duplicate call can end the app
- The backend retains the closure while the ID remains registered; no unregister call exists
- The callback is a synchronous work closure that returns `AppRefreshOutcome`; no `Future`, executor, or async wait is part of this contract
- The backend owns the native task and calls its finish method once after the closure returns
- A normal return maps `Succeeded` or `Failed` to the native result, unless expiry wins the atomic race against the finish claim
- The backend must catch a Rust panic at the native boundary and finish with failure once
- Expiry is a cooperative atomic signal only; it does not stop Rust work or finish the native task
- If the closure sees expiry, it must stop work and return `Failed` promptly; if it does not return, the OS may end the app before a finish call
- No finish call is assured after process kill or abort
- The callback runs on a backend-owned thread or queue; no order or main-thread promise exists

## Boundaries

- One `BGAppRefreshTask` class only in D20/B25
- No `BGProcessingTask`, `BGContinuedProcessingTask`, `BGHealthResearchTask`, UIKit background task, BackgroundAssets, URLSession transfer, notification schedule, or push API
- No C ABI, Swift, app delegate, entitlement, host plist writer, task history, durable task state, or global Rust registry
- D20 has no device or power limits, exact launch time, periodic recurrence, or promise after user force quit

## Checks

- Add or run no tests for this slice
- G19 owns no-default-features portable check, device/Simulator checks, strict Clippy, Rust docs, format, docs-check, zero-Swift, link/import, and diff gates
- Link/import checks do not prove host launch setup, schedule acceptance, OS launch, task time, expiry, callback behavior, or device behavior

## Handoff

Report public names, exact checks, host metadata needs, direct imports, run limits, and open assumptions
