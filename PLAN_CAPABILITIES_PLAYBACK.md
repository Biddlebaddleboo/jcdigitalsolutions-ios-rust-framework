# PLAN_CAPABILITIES_PLAYBACK.md — Workstream D43: HDR Playback Eligibility

## Status

D43 adds the `no_std` `framework-audio::HdrPlaybackEligibility` snapshot for the narrow HDR
eligibility query implemented by B48. It does not implement a player, audio engine, media decode,
asset inspection, or general playback facade. No tests were added or run in this resumed slice.

## Objective

Define a portable value for Apple's point-in-time answer to whether the system reports HDR playback
eligibility. Keep the portable crate independent of Apple types and runtime state.

## Write scope

- `crates/framework-audio/**`
- `docs/capabilities/playback.md`

Root owns workspace and lock integration, CI, capability manifest/counts, shared indexes, and
aggregate validation docs.

## Contract

- `HdrPlaybackEligibility` stores one boolean returned by an explicitly selected backend.
- The value does not name a media item, confirm that an asset contains HDR, report active playback,
  or prove that the display currently shows HDR.
- Keep the crate `#![no_std]`, allocator-free, and free of platform types and unsafe code.
- Do not turn the snapshot into a transport, device-discovery, or entitlement abstraction.

## Validation

- Check the crate with `--no-default-features`, strict Clippy, and rustdoc.
- Do not add or run tests in this resumed pass.
- Record iOS runtime and hardware limits in B48/G42; a portable value check is not HDR parity
  evidence.
