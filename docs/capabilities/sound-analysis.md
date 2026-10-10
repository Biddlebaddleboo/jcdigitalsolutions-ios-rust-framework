# SoundAnalysis built-in classifier recognition

`framework-media::SoundAnalysisSupportSnapshot` stores one copied result for whether the platform
recognizes the built-in SoundAnalysis classifier request. A `true` value means the iOS backend
successfully constructed the built-in version 1 request and its known-classification list was
nonempty. The backend does not create an analyzer, supply audio, or execute the model.

This narrow value does not imply microphone access, model execution, analysis success, or
custom-model support, and does not report device performance or ShazamKit catalog access. See the
[iOS adapter guide](../ios/sound-analysis.md) for the exact query and API floor.
