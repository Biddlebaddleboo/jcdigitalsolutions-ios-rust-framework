# PLAN_IOS_OTHER_AUDIO.md — Workstream B47: AVAudioSession Other-Audio Snapshot

## Objective

Implement one synchronous, non-mutating iOS query for whether any other app is playing audio

## API and bounds

- Use `AVAudioSession.sharedInstance()` and the read-only `isOtherAudioPlaying` getter from the
  typed `objc2-avf-audio` 0.3.2 binding with only its `AVAudioSession` feature
- Return `framework_media::OtherAudioPlaybackSnapshot`; report iOS 6.0 as the API floor
- Keep generated Objective-C calls inside `ios-media`; expose no Objective-C object
- Make no main-thread, permission, entitlement, or microphone-usage-string claim
- Document the broad snapshot semantics, ambient-audio behavior, and Apple recommendation to use
  `secondaryAudioShouldBeSilencedHint` for most mixing decisions

## Exclusions

No session category/mode/activation, recording, microphone, capture, player, Now Playing item,
metadata, command center, remote command, interruption observer, or live source identification

## Status

The adapter is implemented as `ios_media::other_audio_playback_snapshot()` in
`platform/ios/ios-media/src/audio_playback.rs`. `check-audio-playback.sh` passed locally on Xcode
26.6 / iOS SDK 26.5 for the portable no-default/strict-Clippy gate, iOS device and Simulator
compile/strict-Clippy gates, rustdoc, dependency-feature audit, and Release import/symbol surface.
The Apple probe linked but did not run, and no live audio query or passing CI run is claimed. The
separate CoreMedia value adapter remains unchanged
