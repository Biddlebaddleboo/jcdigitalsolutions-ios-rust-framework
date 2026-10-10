# PLAN_VALIDATION_IOS_OTHER_AUDIO.md — Workstream G41: Other-Audio Snapshot Gate

## Completed shared tooling and future validation profiles

R1/R2 PATH executables are complete. Follow `docs/SHARED_TOOLING.md`; this capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`. Continue the existing focused checks specified below, including AVFAudio singleton/getter selectors, exact imports, no-capture/no-Swift guards and deployment metadata. Future coverage should be defined using schema-v1 JSON and an optional bounded Python adapter only if needed; do not remove the current script or its unique negative assertions before verified positive/negative and unavailable-tool failure parity. Static compile or link results do not imply device-runtime behavior. Do not retrieve or patch shared engine source during API work; file `BUG_REPORT_*.md` on suspected engine defects.

## Objective

Gate D42/B47's portable scalar contract and iOS `AVAudioSession.isOtherAudioPlaying` adapter

## Checks

`sh platform/ios/ios-media/check-audio-playback.sh` runs format, portable no-default check and
strict Clippy, device/Simulator check and strict Clippy, rustdoc, dependency-feature audit, a Release
link/import probe, docs-index check, zero-Swift-source check, and `git diff --check`. The audio-only
checks use `--no-default-features` to exclude the default-enabled CoreMedia time adapter and the opt-in
VideoToolbox adapter. The probe requires exactly AVFoundation, Foundation, libobjc, and libSystem imports; checks for the
singleton/getter selectors; rejects VideoToolbox, audio-operation, capture, permission, and
Swift-runtime symbols; and audits deployment metadata. It links but does not run

## Evidence limits

These gates do not query live audio state, determine source identity, test mixing policy, establish
behavior under audio-session changes, or provide Apple parity/performance evidence. The host Xcode
26.6 / iOS SDK 26.5 result is local evidence only and does not qualify the Xcode 27.x CI lane

## Status

The package gate and exact Release import allowlist are wired in CI. The earlier full
`sh platform/ios/ios-media/check-audio-playback.sh` gate passed locally at source-equivalent commit
`1c8553f` (same source as `2a38984` except for the CI workflow), using Xcode 26.6 build 17F113, iOS
SDK 26.5, Rust/Cargo 1.94.1, and pinned tooling 0.1.0; that historical run predates CoreMedia time
feature isolation and imported CoreMedia in the audio-only probe. After isolation, the full B47 gate
passed locally on the same toolchain. Its `--no-default-features` profile rejects `objc2-core-media`,
VideoToolbox, and all Swift runtime dylibs, and the device and Simulator Release probes import exactly
AVFoundation, Foundation, libobjc, and libSystem at minos 12.0 and 14.0. The separate B22
`sh platform/ios/ios-media/check-link-imports.sh` gate also passed on this host with the default-enabled
`core-media-time` feature; it retains its exact `libSystem.B.dylib` import assertion, rejects
`CMTimeMake` and Swift runtime dependencies, and preserves those deployment floors. The hosted Xcode
27 run below requalifies the audio-only B47 gate and the separate default-feature B22 link/import gate.
Both sets of probes link but do not run; this is not
live audio/runtime evidence

## Hosted Xcode 27 static-gate evidence (2026-10-10)

The Xcode 27 lane in [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) passed step `Check other-audio status contract and iOS backend`, which ran `sh platform/ios/ios-media/check-audio-playback.sh` at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. The runner recorded Xcode 27.0 build 27A266a, iPhoneOS and iPhoneSimulator SDK 27.0, and Rust 1.94.1. The script's format, no-default-feature check and strict Clippy, rustdoc, dependency-feature audit, Release link/import probes, docs-index, zero-Swift-source, and diff checks passed. The same run passed step `Link and audit iOS media imports and CMTime layout`, which ran `sh platform/ios/ios-media/check-link-imports.sh` for the default-enabled `core-media-time` profile. That script's exact `libSystem.B.dylib` import, no-`CMTimeMake`, no-Swift-runtime, and deployment-minimum assertions passed for device and Simulator; the probes were not executed. The `crates/framework-media/`, `platform/ios/ios-media/`, workspace manifest/lockfile, `xtask/`, and `.github/workflows/ci.yml` gate inputs are unchanged from the run SHA through current `origin/main` `5b59d9d`.

This is static compile, lint, documentation, and link/import evidence only. The probes were linked and inspected, not executed; live audio state, source identity, mixing policy, and session-change behavior remain unverified.
