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
26.6 / iOS SDK 26.5 remains below the Xcode 27.x plan baseline

## Status

The package gate and exact Release import allowlist are wired in CI. The local gate passed on Xcode
26.6 / iOS SDK 26.5 for device and Simulator targets. Both linked probes import exactly AVFoundation,
CoreFoundation, CoreMedia, Foundation, libobjc, and libSystem; their deployment versions are 12.0
and 14.0. They were not executed, and no CI workflow run is recorded
