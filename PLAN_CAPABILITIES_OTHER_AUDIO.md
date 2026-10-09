# PLAN_CAPABILITIES_OTHER_AUDIO.md — Workstream D42: Other-Audio Snapshot

## Objective

Add a portable point-in-time value for whether another app is playing any audio; do not claim
general Now Playing, media metadata, playback control, or audio capture

## Scope

- Add `framework_media::OtherAudioPlaybackSnapshot` with a Boolean constructor and accessor
- Keep the contract independent of AVFAudio, Objective-C, allocation, and an executor
- Treat the value as stale immediately after it is read; identify no app or media item
- Document that Apple's broad `isOtherAudioPlaying` query includes ambient audio

## Exclusions

- No Now Playing metadata or current-item identity
- No MediaPlayer command center, remote command, player, queue, or playback controls
- No audio session configuration or activation, recording, microphone access, permission request,
  stream, or real-time guarantee
- No assertion that this slice completes row 051's full capability family

## Status

The portable type is implemented in `crates/framework-media/src/lib.rs`, included in the no-std
selected-API fixture, and documented in `docs/capabilities/other-audio.md`. B47 is the separate
AVFAudio iOS adapter in `PLAN_IOS_OTHER_AUDIO.md`
