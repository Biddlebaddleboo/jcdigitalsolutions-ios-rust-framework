# PLAN_CAPABILITIES_PRIVACY_AUTHORIZATION.md — Workstream D35: App Tracking Status Contract

## Reconciled status — 2026-10-10

D35's portable ATT surface is present in `framework-auth`; this pass closes documentation precision and deterministic coverage gaps without changing D12 local-auth behavior. `AppTrackingAuthorizationStatus` has five distinct values, and `AppTrackingAuthorizationBackend::status` is synchronous and statically dispatched. The iOS adapter maps all four documented Apple states and maps an unrecognized native state to `Unknown`; native status calls, prompts, identifiers, and tracking are outside this workstream's validation.

The portable enum/backend are `#![no_std]`, forbid unsafe code, and add no external dependency. Docs now state Apple's status semantics, calling-app/ATT-only scope, unknown-value preservation, and the limits of `Authorized`. A deterministic fake-backend test forwards all five portable values.

## Objective

Add a no_std portable contract for App Tracking Transparency authorization status only. Do not claim a general privacy-authorization facade.

## Dependencies

- D1 portable API conventions are available
- No third-party dependency is needed by the portable crate
- B40 supplies the iOS status backend in a separate workstream

## Write scope

- `crates/framework-auth/**`
- `docs/capabilities/privacy-authorization.md`

Do not edit the root workspace configuration, capability status manifest, shared docs indexes, CI, aggregate plans, or `tools/xtask`. Keep this worktree isolated from the shared checkout.

## Required contract

- Build as portable `#![no_std]` Rust with no third-party dependencies and no unsafe code
- Expose a status enum for App Tracking Transparency only: `NotDetermined`, `Restricted`, `Denied`, `Authorized`, and `Unknown`
- Expose a synchronous, statically selected backend trait
- State that the status belongs to the calling app and reports only App Tracking Transparency
- Do not add a prompt, authorization request, identifier access, tracking implementation, or status for other privacy features
- Preserve unknown native status values as `Unknown`

## Documentation

Explain each status with Apple-defined semantics and state that this is not general privacy consent, data-access status, advertising-identifier availability, or a guarantee that tracking is lawful or otherwise allowed.

## Validation and handoff

- Add deterministic tests for distinct portable states and a fake backend
- Run focused formatting, portable tests/check, strict Clippy, and `git diff --check`
- Inspect the public API for Apple types, `std`, unsafe code, dynamic dispatch, hidden initialization, and unrelated dependencies
- Report exact checks and note that no prompt or tracking operation was tested

## D35 reconciliation evidence

Results from workstream branch `workstream/d35-att-portable-status`, based on `9760e8c63409fcf3268c80a9969559d6f59940af`:

- Passed: `cargo +1.94.1 fmt --all -- --check`
- Passed: `cargo +1.94.1 test --locked --offline -p framework-auth` (2 unit tests, 0 failures; 0 doc tests)
- Passed: `cargo +1.94.1 check --locked --offline -p framework-auth --no-default-features`
- Passed: `cargo +1.94.1 clippy --locked --offline -p framework-auth --all-targets -- -D warnings`
- Passed audit: `cargo +1.94.1 tree --locked --offline -p framework-auth` shows only the workspace-local `framework-core` dependency; the portable ATT surface has no third-party dependency, `std`, unsafe block, Apple type, or dynamic-dispatch wrapper.
- Passed: `git diff --check`

No iOS runtime status, prompt, user choice, identifier, or tracking operation is exercised by these portable checks. No general privacy-consent or legal/tracking permission conclusion is exposed.
