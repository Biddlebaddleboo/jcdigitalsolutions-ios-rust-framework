# PLAN_CAPABILITIES_PHOTOS.md — Workstream D22: Photos Read/Write Authorization

## Status

D22 portable source and guide are implemented. In isolated temporary workspace `/tmp/photos-ios-check.zHPcdc`, three deterministic tests, no-default-features check, strict Clippy, and rustdoc pass on Rust 1.94.1. The temporary workspace resolved the same package versions as the implementation; root owns the shared lock integration

## Objective

Add a portable, `no_std` contract for Photos read/write authorization status and one explicit authorization request. Keep `Limited` distinct from `Authorized`

## Dependencies

- B27 implements the iOS 14+ PhotoKit backend in `ios-photos`
- No allocator or third-party dependency

## Write scope

- `PLAN_CAPABILITIES_PHOTOS.md`
- `crates/framework-photos/**`
- `docs/capabilities/photos.md`

Root owns workspace and lock integration, CI, canonical capability status, aggregate plans, shared docs indexes, and aggregate validation docs. Do not edit those shared paths

## Contract

- Expose `PhotoLibraryAuthorizationStatus::{NotDetermined, Restricted, Denied, Limited, Authorized, Unknown}`
- Preserve `Limited` as a distinct status; do not treat it as full `Authorized`
- Expose a static `PhotoLibraryAuthorizationBackend` plus a thin `PhotoLibrary<B>` facade
- Synchronous status queries do not prompt
- Authorization requests are explicit, start no earlier than first poll, and return the final platform status
- Dropping a pending request suppresses its Rust result but does not promise native cancellation or UI dismissal
- No executor or `Send` requirement
- Do not enumerate assets, request image data, edit assets, expose identifiers, or claim that an authorization status predicts a later operation's result

## Validation

- Run package format, host check/test, strict Clippy, and rustdoc
- Deterministic fake-backend tests cover Limited preservation, first-poll start, and drop before first poll
- Run `git diff --check`
- Root must add the workspace lock entry before repository-wide `--locked` validation

## Validation record

In isolated workspace `/tmp/photos-ios-check.zHPcdc`, Rust 1.94.1 passes:

- `cargo +1.94.1 fmt --manifest-path crates/framework-photos/Cargo.toml -- --check`
- `cargo +1.94.1 test --locked --offline -p framework-photos` — three tests pass
- `cargo +1.94.1 check --locked --offline -p framework-photos --no-default-features`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p framework-photos -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p framework-photos --no-deps`

No shared workspace or lock file was edited by D22

## Apple API basis

The native status values map from Apple's public `PHAuthorizationStatus` values; `Limited` is value 4 and was introduced with iOS 14. See [PHAuthorizationStatus](https://developer.apple.com/documentation/photos/phauthorizationstatus)
