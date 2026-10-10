# PLAN_CAPABILITIES_OTHER_AUDIO.md — Workstream D42: Other-Audio Snapshot

## Objective

Add a portable point-in-time value for whether another app is playing any audio; do not claim
general Now Playing, media metadata, playback control, or audio capture

## Scope

- Add `framework_media::OtherAudioPlaybackSnapshot` with a Boolean constructor and accessor
- Keep the contract independent of AVFAudio, Objective-C, allocation, and an executor
- Treat the value as stale immediately after it is read; identify no app or media item
- Document that Apple's broad `isOtherAudioPlaying` query includes ambient audio

## Exclusions

- No Now Playing metadata or current-item identity
- No MediaPlayer command center, remote command, player, queue, or playback controls
- No audio session configuration or activation, recording, microphone access, permission request,
  stream, or real-time guarantee
- No assertion that this slice completes row 051's full capability family

## Status

The portable type is implemented in `crates/framework-media/src/lib.rs`, included in the no-std
selected-API fixture, and documented in `docs/capabilities/other-audio.md`. B47 is the separate
AVFAudio iOS adapter in `PLAN_IOS_OTHER_AUDIO.md`

## D42 Completion Evidence — 2026-10-10

The D42 contract was already present at the start of this workstream; no product source or guide
change was needed. `OtherAudioPlaybackSnapshot` is a `Copy` value backed by one `bool`, with
`const` construction and access, in the `#![no_std]` `framework-media` crate, whose manifest has
no dependencies. Rustdoc and the portable guide describe a point-in-time, non-live value that
identifies no app or item. The iOS guide states that Apple's broad `isOtherAudioPlaying` query
includes ambient-category audio and that the result can change immediately after the call. The
value exposes no Now Playing, media-control, capture, or stream operation. The iOS guide excludes
session activation/configuration, microphone access, permission requests, and real-time guarantees.

The existing selected-API fixture at
`tools/xtask/fixtures/no-std-link-probe/src/lib.rs::check_framework_media` constructs
`OtherAudioPlaybackSnapshot` and calls `is_other_audio_playing()`. Its C harness invokes that check.
`cargo xtask no-std-link-probe` passed on the 47-entry registry on 2026-10-10: the host probe ran
with exit code 0, and arm64 iOS device and Simulator artifacts linked and passed their import and
symbol audits with Xcode 26.6 / SDK 26.5. The `framework-media` selected API check is in the probe
object without separate archive-object attribution. Apple target artifacts were not executed;
this evidence is not device/Simulator runtime proof, full application linkage proof, or a claim
that every `framework-media` path is linked.

Focused checks passed with Rust 1.94.1:

- `cargo check -p framework-media --no-default-features --offline --locked`
- `cargo clippy -p framework-media --lib --no-default-features --offline --locked -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc -p framework-media --no-default-features --no-deps --offline --locked`
- `cargo fmt --all -- --check`
- `cargo xtask no-std-link-probe`

The repository-pinned `ios-rust-build` and `ios-rust-validate` 0.1.0 tools were installed from the
checked-in archive; both report source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`. The
validator lists only the four unrelated registered profiles, so no D42-specific validator profile
or engine check applies. This workstream does not change or expand B47 or claim completion of
row 051's full capability family.
