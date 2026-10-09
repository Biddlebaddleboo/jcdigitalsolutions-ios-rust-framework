# PLAN_CAPABILITIES_PRIVACY_AUTHORIZATION.md — Workstream D35: App Tracking Status Contract

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
