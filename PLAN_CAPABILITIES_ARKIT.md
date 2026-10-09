# D49 — Portable ARKit world-tracking support value

## Objective

Add one framework-owned, portable boolean value for the result of an iOS world-tracking support
query. This does not claim portable AR execution; it keeps the system result independent of ARKit
classes and session state.

## Scope

- `crates/framework-maps/**`
- `docs/capabilities/arkit.md`
- row 094 only in `docs/capabilities/capability-status.json` plus its summary counters

The orchestrator owns integration with other capability changes and shared support-count prose.
Do not edit other matrix rows, shared root plans, platform backends, Swift sources, tests, or
Cargo workspace declarations.

## Contract

- Expose `WorldTrackingSupport(bool)` with a system constructor and `is_supported()` accessor.
- Keep the crate `#![no_std]`, unsafe-free, and free of platform types.
- Interpret the boolean only as the platform's support result for a world-tracking configuration.
- Do not model camera authorization, active tracking, environment quality, or session lifecycle.

## Acceptance

- The portable contract builds with `--no-default-features` and has complete rustdoc.
- Row 094 classifies the result as a partial portable contract and iOS backend, not complete ARKit
  support.
- The portable guide states the partial scope and links to the iOS backend guide.
- No tests, device runtime behavior, tracking, or performance claims are added.

## Status

Implemented. Local no-std, format, strict Clippy, and documentation evidence is recorded in
`PLAN_IOS_ARKIT.md`; the manifest row is row 094 only. No tests or runtime queries were run.
