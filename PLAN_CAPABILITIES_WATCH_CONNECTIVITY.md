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
