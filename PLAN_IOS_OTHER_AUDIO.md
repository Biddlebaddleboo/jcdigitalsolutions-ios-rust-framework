# PLAN_IOS_OTHER_AUDIO.md — Workstream B47: AVAudioSession Other-Audio Snapshot

## Objective

Implement one synchronous, non-mutating iOS query for whether any other app is playing audio

## API and bounds

- Use `AVAudioSession.sharedInstance()` and the read-only `isOtherAudioPlaying` getter from the
  typed `objc2-avf-audio` 0.3.2 binding with only its `AVAudioSession` feature
- Return `framework_media::OtherAudioPlaybackSnapshot`; report iOS 6.0 as the API floor
- Keep generated Objective-C calls inside `ios-media`; expose no Objective-C object
- Keep the separate VideoToolbox adapter behind `ios-media`'s opt-in `videotoolbox` feature so the
  default audio path does not link VideoToolbox
- Make no main-thread, permission, entitlement, or microphone-usage-string claim
- Document the broad snapshot semantics, ambient-audio behavior, and Apple recommendation to use
  `secondaryAudioShouldBeSilencedHint` for most mixing decisions

## Exclusions

No session category/mode/activation, recording, microphone, capture, player, Now Playing item,
metadata, command center, remote command, interruption observer, or live source identification

## Status

The adapter is implemented as `ios_media::other_audio_playback_snapshot()` in
`platform/ios/ios-media/src/audio_playback.rs`. The full
`sh platform/ios/ios-media/check-audio-playback.sh` gate passed locally at source-equivalent commit
`1c8553f` (same source as `2a38984` except for the CI workflow): Xcode 26.6 build 17F113, iOS SDK
26.5, Rust/Cargo 1.94.1, and pinned tooling 0.1.0. No checks were skipped. The verified deployment
floors are iOS 12.0 for device and iOS 14.0 for Simulator; the default feature graph excludes
VideoToolbox. The Apple probe linked but did not run, so this records no live audio query or runtime
behavior. The separate CoreMedia value adapter remains unchanged
