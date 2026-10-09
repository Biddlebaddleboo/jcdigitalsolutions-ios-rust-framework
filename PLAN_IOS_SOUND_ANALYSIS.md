# PLAN_IOS_SOUND_ANALYSIS.md — Workstream B46: SoundAnalysis Classifier Query

## Objective

Implement D41's built-in classifier recognition snapshot without audio analysis.

## Bounded API

- On iOS 15.0+, use `SNClassifierIdentifierVersion1` to initialize `SNClassifySoundRequest`
- Read only the request's `knownClassifications` and map successful nonempty labels to supported
- Use `objc2-sound-analysis` 0.3.2 with default features disabled and only `SNClassifySoundRequest` / `SNTypes`
- Do not enable `block2`, audio-input, Core Media, Core ML, analyzer, result, or ShazamKit features
- Do not create analyzers, supply audio, access a microphone, request permission, or claim analysis success

## Validation

Run `platform/ios/ios-sound-analysis/check.sh` for portable and iOS target checks, strict Clippy, rustdoc, feature review, and Release import/symbol audit. Link evidence does not establish model or audio runtime behavior.
