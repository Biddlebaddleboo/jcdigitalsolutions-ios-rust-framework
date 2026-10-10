# PLAN_CAPABILITIES_REPLAYKIT.md — Workstream D40: ReplayKit Availability

## Objective

Add a portable scalar for the legacy ReplayKit screen-recorder availability value. This is an additive value in the existing `framework-media` crate and does not change D17's finite `MediaTime` contract.

## Required contract

- Store one copied Boolean in a `no_std`, allocation-free value
- State that availability is point-in-time, can change, and does not imply consent or future recording success
- Do not expose capture, recording, broadcast, picker, permission, microphone, or camera operations
- State that Apple marks ReplayKit `isAvailable` deprecated and recommends ScreenCaptureKit; do not imply this slice implements the replacement

## Validation

Run the portable no-default check, strict Clippy, and rustdoc through `platform/ios/ios-replaykit/check.sh`. No live availability value is claimed.

## Write scope

- Additive `ReplayKitAvailability` value in `crates/framework-media/**`
- `docs/capabilities/replaykit.md`

The orchestrator owns workspace integration, the canonical matrix, CI, and aggregate-plan updates.

## D40 reconciliation evidence (2026-10-10)

The existing portable value in `crates/framework-media` and `docs/capabilities/replaykit.md`
already meet this workstream's contract; no product source or capability-documentation change was
needed. `framework-media` is `#![no_std]`, forbids unsafe code, declares no dependencies, and does
not use `alloc`. `ReplayKitAvailability` stores one copied `bool`, with a `const` constructor and
getter. Its API docs identify a point-in-time value that may change, does not establish consent or
future recording success, and covers only the legacy ReplayKit recorder. The capability guide
states that Apple deprecates `RPScreenRecorder.isAvailable` and recommends ScreenCaptureKit's
content-sharing picker availability; it does not claim that this slice implements that replacement.

The existing B45/iOS adapter remains read-only: it queries `RPScreenRecorder.isAvailable` without
starting capture or another operation. D17 `MediaTime` is unchanged. This reconciliation does not
add capture, recording, broadcast, picker, permission, microphone, or camera APIs and does not
change the canonical capability matrix.

`sh platform/ios/ios-replaykit/check.sh` passed with exit code 0 on Xcode 26.6 (17F113), iOS SDK
26.5, and Rust/Cargo 1.94.1. It covered formatting; the portable no-default check, strict Clippy,
and rustdoc; host and `aarch64-apple-ios` / `aarch64-apple-ios-sim` checks and strict Clippy; iOS
adapter rustdoc; dependency-feature and linked-import gates; `cargo xtask docs-check`;
`cargo xtask zero-swift-source`; and `git diff --check`. The pinned `ios-rust-build` and
`ios-rust-validate` 0.1.0 tools were installed from the repository archives; both report source SHA
`2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. The validator list has no ReplayKit profile, so no
`ios-rust-validate --capability` check applies. The package gate runs no tests and makes no live
availability claim. It does not prove runtime behavior on a device or Simulator; Xcode 27
qualification and physical-device runtime remain unverified.
