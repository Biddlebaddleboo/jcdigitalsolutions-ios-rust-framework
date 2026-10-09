# PLAN_VALIDATION_IOS_MEDIA_AUTHORIZATION.md — Workstream G30: Media Authorization Gates

## Objective

Add package-local host and iOS target gates for D31 and B36 without changing shared CI or validation indexes.

## Write scope

- `PLAN_VALIDATION_IOS_MEDIA_AUTHORIZATION.md`
- `crates/framework-media-authorization/check.sh`
- `platform/ios/ios-media-authorization/check.sh`
- `platform/ios/ios-media-authorization/check-surface.sh`

Root owns CI, aggregate validation docs, indexes, workspace metadata, and lockfile integration. Do not edit those paths.

## Required gates

- D31 host no-default-feature check, deterministic tests, strict all-target Clippy, rustdoc, and formatting
- B36 host mapper tests; iOS device and simulator checks and strict Clippy for both Apple targets; iOS-target rustdoc and formatting
- Source-surface guard rejects permission request, session/input, device capture, sample, or audio-engine APIs
- `git diff --check`; docs/index/Swift checks may run but shared index files remain root-owned

## Status

G30 is complete in isolated worktree `/Users/john/Projects/.worktrees/jcdig-media-auth-d31` on
`workstream/capabilities-media-auth-d31`. On Rust 1.94.1, the package-local scripts pass:

- `./crates/framework-media-authorization/check.sh` — no-default-feature host check, two tests,
  strict Clippy, and rustdoc
- `./platform/ios/ios-media-authorization/check.sh` — two host mapper tests; iOS device and
  simulator checks; host/device/simulator strict Clippy; iOS-target rustdoc; status-only source
  guard

`cargo fmt --manifest-path crates/framework-media-authorization/Cargo.toml -- --check`,
`cargo fmt --manifest-path platform/ios/ios-media-authorization/Cargo.toml -- --check`,
`cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check` also pass. Xcode
26.6 build 17F113 and SDK 26.5 are below the repository's Xcode 27.x baseline. These gates do not
establish app runtime, authorization state, prompt behavior, device use, or capture behavior.

The locked package scripts ran with a temporary resolved `Cargo.lock` containing
`objc2-av-foundation 0.3.2`; that lock edit was restored before handoff. Root lock integration is
required before the scripts can run with `--locked` in the integrated checkout.
