# PLAN_VALIDATION_IOS_MEDIA_AUTHORIZATION.md — Workstream G30: Media Authorization Gates

## Shared-tooling execution (current)

The current `ios-media-authorization` profile is registered in `tools/validation/specs/validation-v1.json`. Use it for format, host/device/Simulator checks, strict Clippy, rustdoc, source-contract adapter, docs, and policy gates. Schema v1 does not run Cargo tests; the package `check.sh` retains only `cargo +1.94.1 test --locked -p ios-media-authorization`. The required Python adapter owns the complete deterministic source-surface contract, including prohibited capture/audio APIs. Do not infer runtime proof from static checks or inspect shared engine source.

## Objective

Add package-local host and iOS target gates for D31 and B36; route the existing macOS package gate through the registered validator profile without changing CI trigger policy or validation indexes.

## Write scope

- `PLAN_VALIDATION_IOS_MEDIA_AUTHORIZATION.md`
- `crates/framework-media-authorization/check.sh`
- `platform/ios/ios-media-authorization/check.sh`
- `tools/validation/adapters/check_media_authorization.py`

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

## Hosted CI requalification

[GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. At that revision, the `Rust checks (macos-15)` and `Rust checks (xcode-27)` jobs passed `Check camera and microphone authorization package gates`, which ran the then-current `platform/ios/ios-media-authorization/check.sh`. The Xcode 27 job also passed `Clippy all features on Apple host`, `Unit tests`, `Portable no_std checks`, `Rust documentation`, and `Documentation and zero-Swift-source` checks. It recorded Xcode 27.0 build `27A266a`, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. This historical run does not validate the present profile wiring; the current CI runs the registered validator profile on macos-15 and `--all` on xcode-27. These results do not establish app runtime, authorization state, prompt behavior, device use, or capture behavior.
