# PLAN_VALIDATION_IOS_REPLAYKIT.md — Workstream G39: ReplayKit Gates

## Objective

Gate D40/B45's scalar status query without starting capture or recording.

## Required gates

- Formatting, portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator compile plus strict Clippy
- Release device and Simulator link/import audit for ReplayKit, Foundation, libobjc, and libSystem; require Objective-C message dispatch and reject Swift/capture/broadcast imports
- Docs, zero-Swift-source, and whitespace checks
- Feature audit: no `block2`, broadcast features, or `objc2-core-media`

The probe is linked, not run. No live availability value, user-consent result, capture, recording, or broadcast is claimed.
