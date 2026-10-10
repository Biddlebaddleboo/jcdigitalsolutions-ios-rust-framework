# PLAN_VALIDATION_IOS_REPLAYKIT.md — Workstream G39: ReplayKit Gates

## Completed shared tooling and future validation profiles

R1/R2 PATH executables are complete. Follow `docs/SHARED_TOOLING.md`; this capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`. Continue the existing focused checks specified below, including ReplayKit imports, no-broadcast/no-capture and feature-isolation negative assertions. Future coverage should be defined using schema-v1 JSON and an optional bounded Python adapter only if needed; do not remove the current script or its unique negative assertions before verified positive/negative and unavailable-tool failure parity. Static compile or link results do not imply device-runtime behavior. Do not retrieve or patch shared engine source during API work; file `BUG_REPORT_*.md` on suspected engine defects.

## Objective

Gate D40/B45's scalar status query without starting capture or recording.

## Required gates

- Formatting, portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator compile plus strict Clippy
- Release device and Simulator link/import audit for ReplayKit, Foundation, libobjc, and libSystem; require Objective-C message dispatch and reject Swift/capture/broadcast imports
- Docs, zero-Swift-source, and whitespace checks
- Feature audit: no `block2`, broadcast features, or `objc2-core-media`

The probe is linked, not run. No live availability value, user-consent result, capture, recording, or broadcast is claimed.
