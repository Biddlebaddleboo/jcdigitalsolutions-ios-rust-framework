# PLAN_CAPABILITIES_MEDIA.md — Workstream D17: Portable Finite Media Time

## Status

D17 adds a `no_std` finite rational `MediaTime`, exact comparison, four deterministic unit checks, and a portable guide. Host no_std check, strict Clippy, rustdoc, and unit checks pass on Rust 1.94.1. D17 does not add a native media operation or close row 049; B22 covers the CoreMedia value bridge

## Objective

Add one portable finite media-time value for API contracts that need exact rational seconds without float approximation

## Dependencies

- Workstream A foundation
- No third-party crate or allocator
- B22 maps this value to CoreMedia `CMTime` by value

## Write scope

- `PLAN_CAPABILITIES_MEDIA.md`
- `crates/framework-media/**`
- `docs/capabilities/media.md`

Root owns workspace and lock integration, CI, capability manifest and counts, shared indexes, and aggregate validation docs. Do not edit those shared paths

## Contract

- `MediaTime::new(value: i64, timescale: i32)` accepts only a strictly positive timescale and returns `MediaTimeError::InvalidTimescale` otherwise
- The rational value is `value / timescale` seconds. The signed numerator and positive denominator remain as supplied
- `MediaTime::compare`, `Eq`, `Ord`, and `PartialOrd` use exact cross-products in `i128`; all products fit because the factors are at most `i64` and positive `i32`
- No float conversion, approximation, scale conversion, normalization, epoch, invalid, infinite, indefinite, or rounded state exists
- No `Hash` impl exists because equal rational values may keep distinct numerator/denominator pairs
- Keep the crate `no_std`, allocator-free, and free of unsafe code or platform selection

## Boundaries

- No `CMTimeRange`, `CMSampleBuffer`, audio/video payload, playback, capture, codec, timestamp clock, AVFoundation API, permission, or entitlement
- D17 covers finite time only. Keep row 049 partial until a separate sample-payload contract lands
- Do not reuse or amend D16/B21 geometry types

## Checks

- Run `cargo fmt --manifest-path crates/framework-media/Cargo.toml -- --check`
- Run host `cargo check`, strict all-target Clippy, rustdoc, and deterministic unit checks for the package
- `cargo +1.94.1 check --locked --offline -p framework-media --no-default-features` passes
- Tests cover scale validation and exact comparison only, not native media behavior

## Validation record

On Rust 1.94.1, these checks pass

- `cargo +1.94.1 fmt --manifest-path crates/framework-media/Cargo.toml -- --check`
- `cargo +1.94.1 check --locked --offline -p framework-media --no-default-features`
- `cargo +1.94.1 test --locked --offline -p framework-media` — four unit checks pass
- `cargo +1.94.1 clippy --locked --offline --all-targets -p framework-media -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p framework-media --no-deps`

No native runtime or performance claim
