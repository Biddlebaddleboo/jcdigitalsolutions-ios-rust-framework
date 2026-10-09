# iOS SoundAnalysis classifier recognition

`ios-sound-analysis::support_snapshot()` checks whether `SNClassifySoundRequest` can initialize with
`SNClassifierIdentifierVersion1` and exposes a nonempty `knownClassifications` list. It calls no
analyzer, does not process an audio file or stream, and does not access the microphone or request
permission. The query is available on iOS 15.0+; older systems and non-iOS targets return `false`.

This is not general SoundAnalysis support, a custom-model check, an analysis-success guarantee, or
ShazamKit support. The binding is `objc2-sound-analysis` 0.3.2 with default features disabled and
only `SNClassifySoundRequest` and `SNTypes` enabled. `block2`, audio-input, Core Media, Core ML,
analyzer, result, and ShazamKit features are not enabled.

Run `sh platform/ios/ios-sound-analysis/check.sh` for portable and iOS device/Simulator checks,
strict Clippy, rustdoc, Release imports/symbols, docs, and zero-Swift-source checks. Probes are linked,
not run; no live model, microphone, audio-analysis, or ShazamKit behavior is claimed.

## References

- [Apple `SNClassifySoundRequest`](https://developer.apple.com/documentation/soundanalysis/snclassifysoundrequest)
- [Apple classifier initializer](https://developer.apple.com/documentation/soundanalysis/snclassifysoundrequest/init%28classifieridentifier%3A%29)
- [Apple `knownClassifications`](https://developer.apple.com/documentation/soundanalysis/snclassifysoundrequest/knownclassifications?language=objc)
- [Apple audio-stream analysis guide](https://developer.apple.com/documentation/soundanalysis/classifying-sounds-in-an-audio-stream)
- [`objc2-sound-analysis` 0.3.2 bindings](https://docs.rs/objc2-sound-analysis/0.3.2/objc2_sound_analysis/struct.SNClassifySoundRequest.html)
