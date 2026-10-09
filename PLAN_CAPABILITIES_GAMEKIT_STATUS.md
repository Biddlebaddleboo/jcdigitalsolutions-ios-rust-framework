# PLAN_CAPABILITIES_GAMEKIT_STATUS.md — Workstream D50: Local-Player Status Contract

## Objective

Add a minimal portable value and static backend contract for a point-in-time local-player game-service authentication status. This does not implement GameKit as a general service.

## Dependencies

- Foundation A and `framework-core` are integrated
- The contract must not expose GameKit/Apple types
- B55 supplies the iOS Game Center implementation

## Write scope

- `crates/framework-game/**`
- `docs/capabilities/gamekit-status.md`
- the D50 reference in `PLAN_CAPABILITIES.md`
- `framework-game` membership in the portable no_std check list

Do not add player identity, profile data, achievement, leaderboard, invite, matchmaking, multiplayer, save-game, or account-initialization APIs.

## Required contract

- Create `framework-game` as an independently usable `#![no_std]`, allocator-free crate with no third-party dependency and `#![forbid(unsafe_code)]`.
- Expose `LocalPlayerAuthenticationStatus::{Authenticated, NotAuthenticated}` as a Rust-owned value. `NotAuthenticated` includes any backend state where the native service reports false, including a service that has not been initialized; it does not mean that no account exists.
- Expose a static `LocalPlayerAuthenticationBackend` and caller-owned `LocalPlayer<B>` facade with API availability and one synchronous, non-prompting status read. Availability does not probe account state or signed entitlements.
- Require no global runtime, executor, callback, identity, or native object in the portable API.
- State that the result is point-in-time and may become stale immediately; the contract does not request authentication or observe status changes.

## Validation

- `cargo check --locked -p framework-game --no-default-features`
- `cargo clippy --locked -p framework-game --all-targets --no-default-features -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `git diff --check`
- Do not add or run tests in this slice.

## Acceptance boundary

The contract models only a service-reported local-player Boolean. It does not standardize service identity, account existence, authentication initialization, user consent, or any game data API.
