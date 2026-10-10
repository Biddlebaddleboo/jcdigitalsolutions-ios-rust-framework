# PLAN_IOS_APPLICATION_LIFECYCLE.md — Workstream B453

## Objective

Implement the existing `framework_app::ApplicationBackend` contract for canonical row
`001-core-app-ui-application-lifecycle` with a capability-scoped UIKit backend. Preserve the
portable API, zero-Swift rule, public-API-only rule, and no-global-runtime architecture.

## Dependencies

- Integrated portable `framework-app` contract.
- Integrated `ios-runtime::main_thread::MainThread` and
  `ios_runtime::ffi::catch_unwind`.
- No dependency on B81 scene snapshots; B81 remains unchanged.

## Write scope

- `platform/ios/ios-app-lifecycle/**`
- `docs/ios/app-lifecycle.md`
- This focused plan.

Do not edit `crates/framework-app/**`, B81's `platform/ios/ios-app-scenes/**` or
`docs/ios/app-scenes.md`, the capability manifest, root `PLAN.md`, Cargo workspace/lock files,
validation spec, shared validator tooling, or other iOS backends. Root integrates workspace,
manifest, indexes, CI, and the `ios-app-lifecycle` validation profile/adapter.

## Public Rust surface

Add package `ios-app-lifecycle` with `IosApplicationLifecycle::new(MainThread)`. The backend
implements `framework_app::ApplicationBackend` directly and is passed to the existing generic
`framework_app::Application<B>` facade. It adds no portable types or methods, no C ABI, no native
handle escape, and no mandatory application initialization.

## UIKit event and state contract

Construction samples `UIApplication.applicationState` once and registers for these notification
names on the main queue. It queues no initial event.

| UIKit source | `ApplicationState` update | queued `LifecycleEvent` |
| --- | --- | --- |
| `UIApplicationDidBecomeActiveNotification` | `Foreground` | `StateChanged(Foreground)` |
| `UIApplicationWillResignActiveNotification` | `Inactive` | `StateChanged(Inactive)` |
| `UIApplicationWillEnterForegroundNotification` | `Inactive` | `StateChanged(Inactive)` |
| `UIApplicationDidEnterBackgroundNotification` | `Background` | `StateChanged(Background)` |
| `UIApplicationWillTerminateNotification` | `Terminating` | `TerminationRequested` only |

Seed state maps `UIApplicationStateActive`, `UIApplicationStateInactive`, and
`UIApplicationStateBackground` to `Foreground`, `Inactive`, and `Background`. If UIKit returns an
unknown state value, return `ErrorKind::Platform` with the original signed value as
`PlatformErrorCode`; never coerce it to `Unknown`, `Inactive`, or `Background`. Coalesce consecutive
duplicate state notifications. `poll_event` dequeues at most one item and never blocks.

Use a fixed-capacity 16-event FIFO. On overflow, drop the oldest queued event, then append the new
event. There is no portable overflow event or count in `framework_app`; document the possible loss
of older transition history, and preserve the backend's latest semantic state.

`availability()` returns `Availability::Available` after construction. Construction and all trait
methods, callbacks, and `Drop` require the main thread. Retain `MainThread` in the backend so Rust
cannot send the UIKit observer state to another thread. No permission prompt, entitlement, usage
description, user action, or system configuration is needed.

## Observer ownership and safety

Retain all native Notification Center observer tokens in `IosApplicationLifecycle`; remove every
token in `Drop` on the main thread. Closures capture only shared event state and must not capture the
backend or its token collection. Route observer callbacks through the main queue and contain panics
with `ios_runtime::ffi::catch_unwind`; no panic may unwind through Objective-C/Blocks. Do not add a
global observer, callback registry, executor, process-wide service, worker thread, or UIKit delegate
object.

## App aggregate versus scene lifecycle

This implementation uses `UIApplication` state and app-level notifications. On scene-based apps,
Apple documents `UIApplication.applicationState` as the state of the most active scene. The backend
does not name scenes and is not a per-scene event source. B81 remains a separate, synchronous
`UIApplication.connectedScenes` snapshot; it adds no delegate or event subscription. Do not merge
the APIs or alter B81's semantics.

The initial sample is not a `Launching` event and registration does not replay notifications that
occurred before backend construction. `WillTerminate` is best-effort: iOS may suspend or terminate
an app without delivering it. `Terminating` is only a state update paired with
`TerminationRequested`; do not report `Terminated`, which cannot be observed by the ending process.

## Availability and target limits

Use only public UIKit/Foundation APIs. The installed Xcode 26.6 build 17F113, iPhoneOS SDK 26.5
headers mark `UIApplicationState`, `UIApplication.applicationState`,
`UIApplicationDidEnterBackgroundNotification`, and
`UIApplicationWillEnterForegroundNotification` available from iOS 4.0. This is the API-availability
floor. The effective device deployment minimum is iOS 10.0 with pinned Rust 1.94.1; the compiler
warns and clamps lower deployment settings to that minimum. The registered Simulator minimum is
iOS 14.0. The link/import fixture checks both Mach-O deployment values. This package targets iOS
applications only; it is not app-extension-safe because `UIApplication.sharedApplication` is
unavailable to extensions. No claim is made for macOS, Mac Catalyst, tvOS, watchOS, Simulator
runtime behavior, or physical-device notification behavior.

## Deterministic tests

Keep the transition reducer and fixed ring host-testable and deterministic. Unit tests must cover
all native state values and raw unknown-state errors; each notification mapping; duplicate
coalescing; initial-state-without-event behavior; termination state/event limits; FIFO order and
16-entry drop-oldest overflow; observer removal/no callbacks after drop through an injectable
registration seam; and panic containment. Do not use sleeps, app launch, lifecycle UI actions, or a
live Notification Center as a required unit-test oracle.

The host fixture registers a callback with a fake Notification Center, removes its token through
`ObserverOwner` during backend teardown, then invokes the retained callback and verifies weak event
state is no longer live. A shared callback factory also lets the host test prove panic containment;
the iOS target build verifies production wiring to `ios_runtime::ffi::catch_unwind`. Initial seeded
state is explicitly checked to leave the event queue empty.

## Acceptance commands

The package `check.sh` runs the host tests, device and Simulator checks, strict Clippy, rustdoc,
formatting, and source-surface guards. Root then runs the registered `ios-app-lifecycle` profile,
including its device/Simulator link/import fixture, and reports its structured checks without
claiming runtime proof.

```sh
sh platform/ios/ios-app-lifecycle/check.sh
sh platform/ios/ios-app-lifecycle/check-link-imports.sh
cargo +1.94.1 test --locked -p ios-app-lifecycle
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios
cargo +1.94.1 check --locked -p ios-app-lifecycle --target aarch64-apple-ios-sim
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --all-targets --target aarch64-apple-ios -- -D warnings
cargo +1.94.1 clippy --locked -p ios-app-lifecycle --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo +1.94.1 doc --locked --no-deps -p ios-app-lifecycle --target aarch64-apple-ios --document-private-items
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-app-lifecycle
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-app-lifecycle
```

Linking and import inspection may prove target ABI/link requirements only. Do not invoke or launch
the app during validation. No native runtime, real notification, per-scene event, or termination
claim is accepted from compile, link, source-guard, or deterministic unit-test evidence.
