# PLAN_CAPABILITIES_APP_DATA.md — Workstream D1

## Objective

Implement the first bounded slice of Workstream D from `PLAN_CAPABILITIES.md`: a portable application lifecycle contract, sandbox file and preference capability facades, and request/response values for foreground HTTP. This slice must use Foundation A directly and may use the integrated iOS runtime B only through its documented iOS-specific surface.

## Dependencies

- Foundation A is integrated
- iOS runtime B is integrated
- Swift ABI C and Rust replacement E are not prerequisites for portable contracts

## Write scope

- `crates/framework-app/**`
- `crates/framework-files/**`
- `crates/framework-preferences/**`
- `crates/framework-network/**`
- `docs/capabilities/app-data.md`
- `docs/capabilities/capability-status.json`
- `docs/capabilities/README.md`

Do not edit `PLAN_CAPABILITIES.md`, root workspace configuration, Foundation A, `platform/ios/**`, Swift ABI, C bindings, replacement candidates, or validation tooling. Other capability groups require their own named subplans and executors.

## Required contracts

Each crate must be independently usable and compile as a portable `#![no_std]` crate. Use fixed-width semantic identifiers and framework-owned errors. Avoid platform types in portable APIs, runtime registries, dynamic backend lookup, or hidden process initialization. Backend traits must permit static selection and must not require a boxed trait object.

The application crate should expose lifecycle state/events, availability, and an explicit iOS extension point without mirroring UIKit's object graph. Files and preferences must state path/key, byte/string, ownership, and atomicity semantics. Network values must make headers/body and copy/ownership costs explicit; no implicit executor, retries, cookies, or background transfer behavior may be claimed.

The machine-readable status manifest must cover every capability family and capability listed in `PLAN_CAPABILITIES.md`, with classification R/M/B/A/C/X, portability class, minimum iOS version where verified, framework/permission/entitlement requirements where verified, Swift ABI need, parity/performance status, and native-escape availability. Mark unimplemented rows X with a concise reason; do not infer availability or entitlements.

## Validation and handoff

- Add deterministic contract tests only for implemented portable semantics
- Check every crate with `--no-default-features`
- Check that no portable public surface imports Apple types
- Record APIs, changed files, checks/results, deviations, and every X-class capability in the handoff
- Do not claim Workstream D complete; later capability groups remain separate work
