# PLAN_VALIDATION_IOS_OTHER_AUDIO.md — Workstream G41: Other-Audio Snapshot Gate

## Completed shared tooling and future validation profiles

R1/R2 PATH executables are complete. Follow `docs/SHARED_TOOLING.md`; this capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`. Continue the existing focused checks specified below, including AVFAudio singleton/getter selectors, exact imports, no-capture/no-Swift guards and deployment metadata. Future coverage should be defined using schema-v1 JSON and an optional bounded Python adapter only if needed; do not remove the current script or its unique negative assertions before verified positive/negative and unavailable-tool failure parity. Static compile or link results do not imply device-runtime behavior. Do not retrieve or patch shared engine source during API work; file `BUG_REPORT_*.md` on suspected engine defects.

## Objective

Gate D42/B47's portable scalar contract and iOS `AVAudioSession.isOtherAudioPlaying` adapter

## Checks

`sh platform/ios/ios-media/check-audio-playback.sh` runs format, portable no-default check and
strict Clippy, device/Simulator check and strict Clippy, rustdoc, dependency-feature audit, a Release
link/import probe, docs-index check, zero-Swift-source check, and `git diff --check`. Its default
feature graph excludes the opt-in VideoToolbox adapter. The probe requires exactly AVFoundation,
CoreFoundation, CoreMedia, Foundation, libobjc, and libSystem imports; checks for the
singleton/getter selectors; rejects VideoToolbox, audio-operation, capture, permission, and
Swift-runtime symbols; and audits deployment metadata. It links but does not run

## Evidence limits

These gates do not query live audio state, determine source identity, test mixing policy, establish
behavior under audio-session changes, or provide Apple parity/performance evidence. The host Xcode
26.6 / iOS SDK 26.5 result is local evidence only and does not qualify the Xcode 27.x CI lane

## Status

The package gate and exact Release import allowlist are wired in CI. The full
`sh platform/ios/ios-media/check-audio-playback.sh` gate passed locally at source-equivalent commit
`1c8553f` (same source as `2a38984` except for the CI workflow), using Xcode 26.6 build 17F113, iOS
SDK 26.5, Rust/Cargo 1.94.1, and pinned tooling 0.1.0. No checks were skipped. Both linked probes
import exactly AVFoundation, CoreFoundation, CoreMedia, Foundation, libobjc, and libSystem; verified
deployment floors are iOS 12.0 for device and iOS 14.0 for Simulator. The default feature graph
excludes VideoToolbox. The probes linked but were not executed; this is not live audio/runtime
evidence
