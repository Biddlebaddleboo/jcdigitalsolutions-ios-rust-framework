# iOS playback

`ios-playback::IosFilePlayer` creates a public AVPlayer for an existing local media file. Its
`from_local_file`, `play`, `pause`, and `seek_to` use AVPlayer APIs available from iOS 4.0 and
require `objc2::MainThreadMarker`. `as_av_player` lends `&AVPlayer` on the main thread for host
integration with `AVPlayerLayer` or other public AVFoundation APIs. The wrapper owns the player;
the borrow does not transfer ownership.

`volume`, `set_volume`, `is_muted`, and `set_muted` also require `objc2::MainThreadMarker`. Their
AVPlayer properties are available from iOS 7.0; older systems return
`IosPlayerOutputError::NativeApiUnavailable`, so the package's iOS 4.0 base floor is unchanged.
`volume` returns only a finite native value in the inclusive 0.0 through 1.0 range. `set_volume`
rejects non-finite or out-of-range input without a native call or coercion. AVPlayer volume is a
per-player level relative to system volume: 0.0 silences this player and 1.0 is full volume for
this player. It does not read or set system volume, and Apple says not to use it as a user-facing
volume slider. The crate exposes no system-volume UI or programmatic system-volume control.
`is_muted` and `set_muted` access only the player mute flag; they do not report device mute state,
route suppression, or audible output.

The constructor accepts an absolute Rust UTF-8 path, rejects embedded NUL, opens it to verify
readability and regular-file metadata, then passes the unchanged path to `NSURL::fileURLWithPath`.
This is a preflight, not a guarantee that the file remains present or unchanged after return.
AVPlayer asynchronously determines asset readiness and supported media/codecs. Calling `play`
signals a desire to play; it does not guarantee readiness, decoding, audible output, or visible
output. The host owns file lifetime, `AVAudioSession` policy, permissions, UI, and layer attachment.
This API neither prompts for permissions nor reads or changes `AVAudioSession`.

`seek_to` starts `AVPlayer.seekToTime:completionHandler:` immediately and returns a future. Its
`MediaTime` numerator and positive timescale map exactly to `CMTime`; no float conversion or
rounding is used. `SeekOutcome::Completed` reports AVPlayer's uninterrupted completion flag.
`SeekOutcome::NotCompleted` reports its false flag, which can mean there is no current item or the
seek was interrupted or superseded; AVPlayer does not distinguish these causes. Multiple seeks are
allowed and a later seek can interrupt an earlier one. The future borrows the player and is
main-thread-bound; dropping it detaches the Rust result but does not request native cancellation.
Seek completion does not prove that a frame was decoded or displayed, that playback is active, or
that output began. No current-time or duration observation is exposed because native values can be
invalid or indefinite.

`observe_current_item_end` returns `None` when AVPlayer has no current item; otherwise it returns
`IosPlayerItemEndObservation` for the exact item current at registration. The observer retains that
item and does not follow later item replacement. Apple may post
`AVPlayerItemDidPlayToEndTimeNotification` on a thread other than the registration thread; the
framework filters by the observed item and uses synchronous posting-thread delivery only to set an
atomic flag. `has_reached_end` is a pollable, latched snapshot with no app callback. It reports a
notification delivered after registration, not an item already at its end, replay policy, or
displayed/audible output. Dropping the observer removes its Foundation notification token.

`hdr_playback_eligibility()` remains a separate main-thread snapshot of
`AVPlayer.eligibleForHDRPlayback`, available from iOS 13.4. It returns `None` off-main-thread,
below iOS 13.4, or on a non-iOS target. The result concerns system HDR eligibility, not a particular
asset, active playback, or confirmed HDR output.

Validation runs host path and seek-state regressions, host/device/Simulator compilation and Clippy, rustdoc, and
Release link/import checks. None executes the link probe or establishes runtime playback, output,
codec support, or hardware behavior.

Apple APIs: [AVPlayer](https://developer.apple.com/documentation/avfoundation/avplayer),
[AVPlayer.volume](https://developer.apple.com/documentation/avfoundation/avplayer/volume),
[AVPlayer.isMuted](https://developer.apple.com/documentation/avfoundation/avplayer/ismuted),
[AVPlayer.play](https://developer.apple.com/documentation/avfoundation/avplayer/play()),
[AVPlayer.pause](https://developer.apple.com/documentation/avfoundation/avplayer/pause()),
[AVPlayer.seekToTime:completionHandler:](https://developer.apple.com/documentation/avfoundation/avplayer/seek%28to%3Acompletionhandler%3A%29),
[AVPlayerItem.didPlayToEndTimeNotification](https://developer.apple.com/documentation/avfoundation/avplayeritem/didplaytoendtimenotification),
[AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)
