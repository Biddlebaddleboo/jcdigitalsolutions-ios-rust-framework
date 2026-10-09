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

D31 is complete in isolated worktree `/Users/john/Projects/.worktrees/jcdig-media-auth-d31` on
`workstream/capabilities-media-auth-d31`, based on `main` HEAD
`7b5513fa2a4864d21a594cbf1fbd43951427155d`. The public API is `CaptureMedia`,
`MediaAuthorizationStatus`, and the static `MediaAuthorization::authorization_status` contract.

`./crates/framework-media-authorization/check.sh` passes: no-default-feature host check, two unit
tests, strict all-target Clippy, and rustdoc. Camera and microphone remain distinct; the API makes
no device-availability or capture claim. Canonical rows 046 and 047 remain root-owned and partial
for the broader camera/microphone capabilities.

The temporary lock resolution was restored before handoff; root must add the new workspace package
and lock entries during integration.
