# PLAN_CAPABILITIES_MEDIA_AUTHORIZATION.md — Workstream D31: Camera/Microphone Authorization Contract

## Objective

Add one no_std portable status-only contract for camera and microphone authorization. Keep each media kind distinct and preserve unknown native status values without claiming portable capture support.

## Write scope

- `PLAN_CAPABILITIES_MEDIA_AUTHORIZATION.md`
- `crates/framework-media-authorization/**`
- `docs/capabilities/media-authorization.md`

Root owns workspace and lock integration, canonical capability rows, shared indexes, CI, and aggregate plans. Do not edit those paths.

## Contract

- Export `CaptureMedia::{Camera, Microphone}`, `MediaAuthorizationStatus::{NotDetermined, Restricted, Denied, Authorized, Unknown}`, and `MediaAuthorization::authorization_status`
- Keep the crate `#![no_std]`, allocator-free, and free of unsafe code or platform types
- Treat status as a snapshot; `Authorized` does not imply hardware presence, availability, or capture readiness
- Keep camera authorization separate from microphone authorization and Photos authorization
- Do not expose permission requests, capture sessions, device enumeration, sample data, recording, or playback

## Acceptance

- Deterministic tests show Camera and Microphone are distinct and all status variants are distinct
- Host `--no-default-features` check, strict all-target Clippy, rustdoc, and package-local checks pass
- Guide states exact host privacy keys and the status-only boundary
- Rows 046 and 047 remain partial for camera/microphone capture; this work closes only authorization-status sub-scope

## Status

D31 was reconciled in isolated worktree `/private/tmp/jc-d31-media-authorization-fresh` on
`workstream/d31-media-authorization-reconcile`, based on current local `main` HEAD
`9895cd704279ae1d19feb98efc44982113ae90a2`. The earlier status cited base
`7b5513fa2a4864d21a594cbf1fbd43951427155d`, which predates the package; that earlier pass is not
evidence for the current source. The existing implementation and guide satisfy the D31 contract,
so reconciliation required no product-code or guide change.

Current source exports distinct `CaptureMedia::{Camera, Microphone}` kinds and all five distinct
`MediaAuthorizationStatus` variants through the static status-only
`MediaAuthorization::authorization_status` contract. It is `#![no_std]`, forbids unsafe code, has
no allocator dependency or platform types, and exposes no request, session, device-enumeration,
sample, capture, recording, or playback operation. Status is documented as a snapshot;
`Authorized` does not imply device presence, availability, or capture readiness. The guide names
the exact host purpose keys `NSCameraUsageDescription` and `NSMicrophoneUsageDescription`, and
states that the status-only contract neither requests permission nor requires those keys.

Validation on Rust `1.94.1` passed:

- `RUSTUP_TOOLCHAIN=1.94.1 ./crates/framework-media-authorization/check.sh` — formatting, locked
  no-default-feature check, two unit tests, strict all-target Clippy, and rustdoc.
- `cargo +1.94.1 xtask docs-check`.
- `cargo +1.94.1 xtask zero-swift-source`.
- `git diff --check`.

No native authorization query, permission prompt, hardware query, or capture operation was run;
these are outside this portable status-only workstream. Root-owned workspace integration and
canonical rows 046 and 047 were read-only; those rows remain partial for broader camera/microphone
capture support.
