# PLAN_CAPABILITIES_SOUND_ANALYSIS.md — Workstream D41: Built-in SoundAnalysis Request

## Objective

Add a portable scalar for recognition of Apple's built-in version 1 SoundAnalysis classifier request. This is an additive value in `framework-media`; it does not claim SoundAnalysis family or ShazamKit support.

## Required contract

- Keep the value copied, `no_std`, and allocation-free
- Define `true` only as successful request construction plus a nonempty known-label list
- State that the snapshot does not imply microphone access, model execution, analysis success, custom-model support, or ShazamKit catalog access
- Do not create an analyzer, provide audio, access the microphone, or request permission

## Validation

Run portable no-default check, strict Clippy, and rustdoc through `platform/ios/ios-sound-analysis/check.sh`. No model execution or audio query is claimed.

## Write scope

- Additive `SoundAnalysisSupportSnapshot` in `crates/framework-media/**`
- `docs/capabilities/sound-analysis.md`

The orchestrator owns workspace integration, the canonical matrix, CI, and aggregate-plan updates.
