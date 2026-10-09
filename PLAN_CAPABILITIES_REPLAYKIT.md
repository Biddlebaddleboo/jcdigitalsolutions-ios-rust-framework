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
