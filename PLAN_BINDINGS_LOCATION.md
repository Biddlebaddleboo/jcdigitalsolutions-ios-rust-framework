# PLAN_BINDINGS_LOCATION.md — Workstream F15: iOS Location C ABI

## Status

F15 source, header, ABI manifest, guide, and focused check are complete. After root refreshed the F15 lock edges, `sh bindings/c/check-ios-location.sh` passed on 2026-10-08. It passed JSON/shell/format/diff, manifest/header symbol and layout/tag assertions, default/iOS/host dependency-tree isolation, host check/strict Clippy/Release build, device and Simulator check/strict Clippy/Release build, and C11/C++17 host/device/Simulator compile/link probes. `otool` imports matched the manifest; required `_objc_getClass`, `_objc_msgSend`, `CLLocationManager`, and `requestLocation` evidence and archive exports passed. `vtool` reports final probe minos 10.0 for device and 14.0 for Simulator. Probe binaries were not executed; no tests were run

The Core Location API floor remains iOS 9.0, but this Rust arm64 device target requires object minos 10.0; the device gate uses 10.0 and does not establish full-library runtime support on iOS 9.0. The gate emitted a rustc warning that `IPHONEOS_DEPLOYMENT_TARGET` was set to 9.0 while rustc supports a minimum of 10.0 during target checks, although final device link probes report minos 10.0. The warning's source was not isolated; report it as a validation caveat. The Simulator gate reports minos 14.0

The location follow-up aligns the C header and guide with the Rust pointer preconditions and adds static checks for the ABI-manifest pointer contract. Root reran `sh bindings/c/check-ios-location.sh` after these assertions on 2026-10-09; it passed the static contract checks and host/device/Simulator build, Clippy, C11/C++17 link, import, export, and deployment gates. Probe binaries were inspected, not executed; no tests or consumers ran. The gate emitted a rustc warning that `IPHONEOS_DEPLOYMENT_TARGET` was 9.0 while rustc supports a minimum of 10.0; final device probe minos is 10.0 and Simulator minos is 14.0

## Goal

Expose the existing B5 one-shot location backend through an opt-in capability-owned C API. Keep explicit permission request, authorization query, current-location request, readiness, poll, cancellation, drop, and thread semantics truthful. Do not add a universal operation registry or executor

## Read first

- `PLAN_BINDINGS.md`
- `PLAN_BINDINGS_ASYNC.md`
- `PLAN_IOS_LOCATION.md`
- `bindings/c/src/ios_location.rs`
- `platform/ios/ios-location/src/platform.rs`
- `docs/ios/location.md`

## Write scope

- `bindings/c/Cargo.toml` (`ios-location` optional feature/dependency edges only)
- `bindings/c/src/lib.rs` (feature gate and re-exports only)
- `bindings/c/src/ios_location.rs`
- `bindings/c/include/framework_ios_location.h`
- `bindings/c/abi-manifest.json` (F15 entry and direct-import/error/runtime inventory)
- `bindings/c/check-ios-location.sh`
- `docs/bindings/ios-location.md`
- `PLAN_BINDINGS_LOCATION.md`

Do not edit `PLAN_BINDINGS.md`, root CI, shared documentation indexes, B5 backend source, workspace membership, or other capability backends. Cargo.lock remains root-owned

## Contract decisions

- One unique opaque handle owns one pinned `async move` future that owns its `IosLocationBackend`. The future borrows its backend only inside the compiler-generated async state; the C handle is not self-referential
- Expose `framework_ios_location_availability`, explicit non-prompting authorization query, explicit foreground-authorization request, and one-shot current-location start. A current request does not request permission
- Preserve B5 request semantics: the authorization request result is reported only after Core Location reports a raw status different from the status sampled before the request; do not return an already-known status immediately. If no status change is reported, the operation remains pending until cancel or destroy abandons its result; neither promises to dismiss permission UI
- Document the C pointer preconditions from each Rust `# Safety` contract: output storage is valid, aligned, and writable; availability output does not overlap a live handle; a start slot is not live on entry or overlapping live operation storage; poll outputs are disjoint from each other and the unique operation handle; destroy uses the original writable slot; handles are not aliased or raced
- Use a capability-owned readiness-only callback, not a terminal callback. The callback signals that the host should poll; it carries no result and does not own context. C poll consumes the result once
- B5's future stores the waker in its operation-scoped completion cell and wakes it from the manager creation run loop. F15 creates the manager on main and uses a per-operation `Wake` signal that invokes only the host notification callback. It does not poll the future from the waker or need a shared scheduler
- The callback must not unwind or re-enter F15. It may run before start returns when the first poll finds a result, or during a poll if a synchronous wake occurs. A host that receives it schedules a later main-thread poll
- All F15 calls and handle operations run on main. Off-main destroy returns `FRAMEWORK_STATUS_UNAVAILABLE` before reading or changing the original handle slot. Cancel succeeds only before readiness is signaled and before a terminal result exists; after either, it returns `FRAMEWORK_STATUS_NOT_FOUND`. A ready result can be consumed once, and later polls return `FRAMEWORK_STATUS_NOT_FOUND`. Successful cancel drops the future on main; current-location drop requests native cancellation. An authorization prompt already shown may remain visible
- Reject malformed outputs and non-finite/negative accuracy before acceptance. Initialize valid outputs. On success, `out_ready` is zero for pending or one for a result; `FrameworkIosLocationResultV1.status` carries the operation outcome and native code when available
- Do not claim location freshness, requested accuracy, time-to-fix, live permission, GPS, background, or runtime behavior

## Required validation

Run `sh bindings/c/check-ios-location.sh`. The gate must parse JSON and shell, check C/Rust symbol and fixed-tag parity, assert the `FrameworkIosLocationResultV1` 64-bit layout, inspect default/feature/host dependency trees, check host stubs, run non-test `cargo check`, strict Clippy, and Release builds for `framework-c-api` on host, iOS device, and iOS Simulator, and compile/link C11 and C++17 probes without executing them. It must compare direct framework imports, required Core Location symbols, forbidden unrelated/Swift/Python imports, archive exports, and deployment `minos` with the manifest

Also run scoped Rust format and `git diff --check`. Do not run tests

## Limits and handoff

The Core Location API floor is iOS 9.0 from B5's `requestLocation()`; `requestWhenInUseAuthorization()` is available from iOS 8.0. The installed Rust arm64 iOS target builds objects with minos 10.0, so the device link gate uses minos 10.0; the Simulator link gate uses minos 14.0. Compile/link checks do not prove full-library runtime support on iOS 9.0, prompt behavior, location delivery, timeout behavior, callback runtime, cancellation races, or physical-device behavior

After focused checks pass, root owns Cargo.lock reconciliation, CI, aggregate binding status, and documentation index integration
