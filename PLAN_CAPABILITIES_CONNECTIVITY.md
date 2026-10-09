# PLAN_CAPABILITIES_CONNECTIVITY.md — Workstream D15: Informational Network Path Snapshot

## Status

The portable contract, deterministic tests, guide, 21-crate `no_std` audit, host/device/Simulator
no_std link probe, shared indexes, and capability manifest are integrated. The package check,
strict Clippy, six unit tests, package/workspace rustdoc, workspace check/Clippy/tests, and shared
docs/zero-Swift gates pass on Rust 1.94.1. B18's separate iOS backend and G12 device/Simulator
compile/Clippy/link-import gates also pass locally. No live path or cancellation runtime evidence
exists; the linked probes were not run. The installed Xcode 26.6 / iOS SDK 26.5 exposes the public
Network.framework path monitor C API from iOS 12.0. This slice is informational only; it must not
gate, suppress, or promise the success of a request.

## Objective

Add a small portable contract for one observed local network-path snapshot. Keep it separate from
D1 foreground HTTP, endpoint reachability, and low-level connection/listener APIs.

## Dependencies

- Foundation A and `framework-core` are integrated
- D1 `framework-network` remains unchanged; its request and response contracts do not depend on
  this snapshot
- B18 implements the separate iOS backend in `PLAN_IOS_CONNECTIVITY.md`

## Write scope

- `PLAN_CAPABILITIES_CONNECTIVITY.md`
- `crates/framework-connectivity/Cargo.toml`
- `crates/framework-connectivity/src/lib.rs`
- `docs/capabilities/connectivity.md`

Do not edit root Cargo configuration, `Cargo.lock`, `tools/xtask`, the capability manifest,
`PLAN_CAPABILITIES.md`, shared capability indexes, iOS backends, `framework-network`, bindings, or
other portable contracts. The orchestrator owns workspace, lockfile, and status-manifest
integration. Do not add Swift source, endpoint checks, or a `Send` bound.

## Portable contract

- Add an independently usable `#![no_std]` crate named `framework-connectivity`; use only
  `framework-core` and no heap allocation or third-party dependency
- Define `NetworkPathStatus` with `Unknown`, `Satisfied`, `Unsatisfied`, and `Satisfiable` states;
  `Unknown` represents an invalid/unavailable native status, not a claim that the network is down
- Define a copyable `NetworkPathSnapshot` that exposes only the status; do not expose Apple path,
  interface, queue, or connection types
- Define a static `NetworkPathBackend` with a concrete associated future and a thin generic
  `Connectivity<B>` facade. Do not use dynamic dispatch, a registry, hidden initialization, or a
  required executor; do not require `Send`
- `Connectivity::current_path()` requests one snapshot. Native work starts no earlier than the
  returned future's first poll. The first non-null native path callback completes the request,
  including a callback whose invalid native status maps to `Unknown`; no update deadline is
  promised
- The backend's associated future owns callback state, detachment/invalidation, and native
  cancellation. Dropping the facade future drops that backend future, which must detach or
  invalidate callback state before requesting native cancellation when available. A generic
  `framework_async::OperationFuture` only unregisters its waker and is not sufficient by itself
  to own this native lifecycle. Late or duplicate callbacks must not access freed future memory,
  wake a detached task, or publish a second result
- Preserve the `framework_core::ErrorKind` category and optional native code through the portable
  error type
- Define `Satisfied` only as a currently observed path that the backend reports can establish a
  connection. It does not prove Internet access, DNS, captive-portal clearance, endpoint
  reachability, or success of any request. `Satisfiable` is a separate native status and must not
  be collapsed into `Satisfied`
- Document that the snapshot is advisory and subject to change immediately. Callers must attempt
  their actual operation and handle its result; this API must not gate or disable requests
- Do not add interface enumeration, link cost/constrained flags, a continuous event stream,
  endpoint probes, low-level sockets/listeners, DNS, or request policy

## Apple API basis and boundaries

Apple documents `NWPathMonitor` as an observer that reports path changes and provides an initial
current-path update after start. Its public C counterpart is `nw_path_monitor_t`. `NWPath.Status`
distinguishes invalid, satisfied, unsatisfied, and satisfiable states. The portable status maps
those four values without claiming the route reaches a particular destination. The base monitor
and status APIs have an iOS 12.0 floor in the inspected SDK. See Apple's [NWPathMonitor](https://developer.apple.com/documentation/network/nwpathmonitor),
[NWPath status](https://developer.apple.com/documentation/network/nwpath/status-swift.enum), and
[Network path monitor C API](https://developer.apple.com/documentation/network/nw_path_monitor_t?language=objc).

Apple deprecates `SCNetworkReachability` preflight checks because network conditions change too
often for them to be accurate; its guidance is to attempt the connection instead. Use URLSession's
`waitsForConnectivity` when the request itself should wait. This workstream does not add or use
`SCNetworkReachability` and does not alter D1 URLSession behavior. See Apple's
[SCNetworkReachability guidance](https://developer.apple.com/documentation/systemconfiguration/scnetworkreachability-g7d?language=objc)
and [URLSession waitsForConnectivity](https://developer.apple.com/documentation/foundation/urlsessionconfiguration/waitsforconnectivity).

## Validation and handoff

- Add deterministic portable tests for each path status, the fake-backend snapshot, first-poll
  start, pending-drop detachment/cancellation, and exactly-once callback behavior, including an
  `Unknown` first callback
- Root integration checks passed: `cargo +1.94.1 fmt --all -- --check`,
  `cargo +1.94.1 check --workspace --locked --offline`,
  `cargo +1.94.1 clippy --workspace --all-targets --locked --offline -- -D warnings`,
  `cargo +1.94.1 test --workspace --locked --offline`,
  `cargo +1.94.1 doc --workspace --no-deps --locked --offline`,
  `cargo +1.94.1 xtask no-std-check` (21 crates),
  `cargo +1.94.1 xtask no-std-link-probe` (host, iOS device, and Simulator), and
  `cargo +1.94.1 --locked --offline xtask docs-check` (including zero-Swift-source)
- Inspect the public API for `std`, allocation, dynamic dispatch, platform types, endpoint
  reachability claims, or a request-gating helper
- Report exact symbols, status mapping, callback cancellation, checks, and unverified assumptions;
  do not edit shared capability totals
