# PLAN_CAPABILITIES_WATCH_CONNECTIVITY.md — Workstream D38: Watch Connectivity Session Support

## Objective

Add a portable value for whether a platform can provide a Watch Connectivity session object. This does not model a paired watch or a communication channel.

## Required contract

- Expose only `WatchConnectivitySupport::{Supported, Unsupported}` and a static backend query
- Require no allocation, global registry, dynamic dispatch, or executor
- State that platform support does not report pairing, counterpart-app installation, activation, reachability, or communication success
- Keep all WatchConnectivity types out of the portable crate

## Validation

Run the portable no-default check, strict Clippy, rustdoc, docs, zero-Swift-source, and diff gates. No watch communication or pairing claim is permitted.

## D38 Evidence (2026-10-10)

- Reconciled the portable API, iOS adapter, and both guides against this contract. The existing API exposes only `WatchConnectivitySupport::{Supported, Unsupported}` and the static `WatchConnectivityBackend::session_support()` query. The portable crate is `no_std`, has no dependencies, and contains no WatchConnectivity types; no product-file change was needed.
- `platform/ios/ios-watch-connectivity/scripts/check.sh` passed with Rust 1.94.1 and Xcode 26.6 (17F113). This ran package formatting, the portable `--no-default-features` check, strict Clippy, rustdoc, iOS device and simulator checks, native import/source guards, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`.
- Pinned `ios-rust-build` and `ios-rust-validate` 0.1.0 were installed and version-checked; D38 has no registered validator profile or native-build spec.
- No watch runtime, pairing, activation, reachability, or communication behavior was exercised or is claimed.
