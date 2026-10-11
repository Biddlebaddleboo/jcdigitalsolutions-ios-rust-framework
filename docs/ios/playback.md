# iOS playback

`ios-playback::IosFilePlayer` creates a public AVPlayer for an existing local media file. Its
`from_local_file`, `play`, and `pause` operations use AVPlayer APIs available from iOS 4.0 and
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

`hdr_playback_eligibility()` remains a separate main-thread snapshot of
`AVPlayer.eligibleForHDRPlayback`, available from iOS 13.4. It returns `None` off-main-thread,
below iOS 13.4, or on a non-iOS target. The result concerns system HDR eligibility, not a particular
asset, active playback, or confirmed HDR output.

Validation runs host path regressions, host/device/Simulator compilation and Clippy, rustdoc, and
Release link/import checks. None executes the link probe or establishes runtime playback, output,
codec support, or hardware behavior.

Apple APIs: [AVPlayer](https://developer.apple.com/documentation/avfoundation/avplayer),
[AVPlayer.play](https://developer.apple.com/documentation/avfoundation/avplayer/play()),
[AVPlayer.pause](https://developer.apple.com/documentation/avfoundation/avplayer/pause()),
[AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)
