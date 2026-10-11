# iOS playback

`ios-playback::IosFilePlayer` creates a public AVPlayer for an existing local media file. Its
`from_local_file`, `play`, `pause`, and `seek_to` use AVPlayer APIs available from iOS 4.0 and
require `objc2::MainThreadMarker`. `as_av_player` lends `&AVPlayer` on the main thread for host
integration with `AVPlayerLayer` or other public AVFoundation APIs. The wrapper owns the player;
the borrow does not transfer ownership.

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

`hdr_playback_eligibility()` remains a separate main-thread snapshot of
`AVPlayer.eligibleForHDRPlayback`, available from iOS 13.4. It returns `None` off-main-thread,
below iOS 13.4, or on a non-iOS target. The result concerns system HDR eligibility, not a particular
asset, active playback, or confirmed HDR output.

Validation runs host path and seek-state regressions, host/device/Simulator compilation and Clippy, rustdoc, and
Release link/import checks. None executes the link probe or establishes runtime playback, output,
codec support, or hardware behavior.

Apple APIs: [AVPlayer](https://developer.apple.com/documentation/avfoundation/avplayer),
[AVPlayer.play](https://developer.apple.com/documentation/avfoundation/avplayer/play()),
[AVPlayer.pause](https://developer.apple.com/documentation/avfoundation/avplayer/pause()),
[AVPlayer.seekToTime:completionHandler:](https://developer.apple.com/documentation/avfoundation/avplayer/seek%28to%3Acompletionhandler%3A%29),
[AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)
