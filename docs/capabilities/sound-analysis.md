# SoundAnalysis built-in classifier recognition

`framework-media::SoundAnalysisSupportSnapshot` stores one copied result for whether the platform
recognizes the built-in SoundAnalysis classifier request. The iOS backend constructs the version 1
request and reads its known-classification labels; it does not create an analyzer or supply audio.

This narrow value does not report microphone access, custom-model support, analysis success, device
performance, or ShazamKit catalog availability. See the [iOS adapter guide](../ios/sound-analysis.md)
for the exact query and API floor.
