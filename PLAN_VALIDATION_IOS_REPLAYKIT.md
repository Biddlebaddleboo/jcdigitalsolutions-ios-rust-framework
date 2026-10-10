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

## 2026-10-10 hosted CI recheck

[CI run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431)
completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`; the Ubuntu
24.04, macOS 15, and xcode-27 jobs all passed. The ReplayKit gate
(`sh platform/ios/ios-replaykit/check.sh`, step 176) passed on both Apple jobs; its macOS-only
step was skipped on Ubuntu. The xcode-27 job recorded Xcode 27.0 build `27A266a`, iPhoneOS SDK
27.0, and iPhoneSimulator SDK 27.0. The run SHA is an ancestor of current `main`
(`f7197e7b48bb35136f308792ae51617011e8e619`); the targeted plan, CI workflow, Cargo
manifests/lockfile, and ReplayKit capability paths are unchanged since the run.

This records hosted static compile/lint, feature-isolation, and link/import checks only. The probe
was not run; no live availability, consent, capture, recording, or broadcast behavior is established.
