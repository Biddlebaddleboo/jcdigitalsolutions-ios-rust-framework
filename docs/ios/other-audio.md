# iOS Other-Audio Snapshot

`ios_media::other_audio_playback_snapshot()` reads
`AVAudioSession.sharedInstance().isOtherAudioPlaying` and returns a portable
`framework_media::OtherAudioPlaybackSnapshot`. The API floor is iOS 6.0

```rust
use ios_media::other_audio_playback_snapshot;

let snapshot = other_audio_playback_snapshot();
let other_audio_is_playing = snapshot.is_other_audio_playing();
```

This synchronous query does not set or activate the app's audio session, access a microphone,
request permission, or expose the current item or its source. Apple defines the value to include any
other app's audio, including ambient-category audio, and recommends
`secondaryAudioShouldBeSilencedHint` for most mixing decisions because it is narrower. The result
can change immediately after the call; it is not general Now Playing status, playback control,
capture support, or a real-time guarantee

The API uses the typed `objc2-avf-audio` 0.3.2 `AVAudioSession` binding with the `AVAudioSession`
feature only. The shared session is documented as a singleton; the installed iOS SDK marks the
class sendable. No main-thread dispatch, microphone usage string, entitlement, or live audio
operation is part of this query. Device/Simulator compilation and import checks do not establish a
live audio result or Apple parity
