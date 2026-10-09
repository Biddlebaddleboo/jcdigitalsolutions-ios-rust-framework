# Other Audio Snapshot

`framework_media::OtherAudioPlaybackSnapshot` stores one point-in-time Boolean for whether any
other app is playing audio

```rust
use framework_media::OtherAudioPlaybackSnapshot;

let snapshot = OtherAudioPlaybackSnapshot::new(true);
assert!(snapshot.is_other_audio_playing());
```

The portable value identifies no app or audio item, does not report this app's playback, and has no
media-control operation. It is not a live subscription; query the platform adapter again for a new
snapshot
