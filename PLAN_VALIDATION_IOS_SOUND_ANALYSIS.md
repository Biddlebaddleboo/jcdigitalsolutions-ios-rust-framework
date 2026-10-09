# PLAN_VALIDATION_IOS_SOUND_ANALYSIS.md — Workstream G40: SoundAnalysis Gates

## Objective

Gate D41/B46's classifier-recognition status query without creating an analyzer or processing audio.

## Required gates

- Formatting, portable no-default check, strict Clippy, and rustdoc
- iOS device and Simulator compile plus strict Clippy
- Release device and Simulator link/import audit for SoundAnalysis, Foundation, libobjc, and libSystem; require Objective-C dispatch and reject Swift, analyzer, microphone-input, and ShazamKit imports
- Feature review that confirms the absence of `block2`, audio-input, Core Media, Core ML, analyzer, and ShazamKit features
- Docs, zero-Swift-source, and whitespace checks

The probe is linked, not run. No live classifier recognition, microphone, audio-analysis, or ShazamKit behavior is claimed.
