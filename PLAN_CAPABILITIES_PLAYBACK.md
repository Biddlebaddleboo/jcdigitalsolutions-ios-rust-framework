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

## Resumed-pass evidence — 2026-10-10

The scoped implementation already meets the D43 contract: `crates/framework-audio` is `#![no_std]`,
forbids unsafe code, and exposes only the boolean `HdrPlaybackEligibility` snapshot with a const
constructor and accessor. `docs/capabilities/playback.md` excludes media-item, asset, active-playback,
player, and actual display-output claims. No source or capability-doc change was needed.

The following host checks pass on Rust 1.94.1:

- `cargo check --package framework-audio --no-default-features`
- `cargo clippy --package framework-audio --no-default-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --package framework-audio --no-default-features --no-deps`

Pinned shared tools 0.1.0 were installed and version-checked from source SHA
`2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`; no D43 validator profile is registered. No tests were
added or run. These host checks do not establish iOS runtime, player, decode, HDR display-output, or
hardware parity; B48/G42 remain the source for their recorded platform-gate and runtime limits.
