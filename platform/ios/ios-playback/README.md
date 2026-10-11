# iOS playback capability snapshot

`ios-playback` exposes `IosFilePlayer` for main-thread local-file AVPlayer creation, play/pause,
per-player volume and mute controls, and asynchronous seek completion.
It also exposes `hdr_playback_eligibility()`, a separate main-thread query of
`AVPlayer.eligibleForHDRPlayback`. AVPlayer local-file creation and play/pause are available from
iOS 4.0; the HDR property remains iOS 13.4+ and returns `None` below its floor. The crate links
AVFoundation through `objc2-av-foundation` 0.3.2 and CoreMedia through `objc2-core-media` 0.3.2.

`seek_to` maps finite `MediaTime` values to exact `CMTime` rationals and returns AVPlayer's
asynchronous completion result. `NotCompleted` does not distinguish an absent current item from a
seek interrupted or superseded by another operation. Completion is not proof of a decoded or
displayed frame; dropping the future detaches Rust result interest without requesting cancellation.

`volume`, `set_volume`, `is_muted`, and `set_muted` require a main-thread marker and use typed
AVPlayer bindings. These properties have an iOS 7.0 floor; runtime guards preserve the existing
iOS 4.0 package floor. The volume setter rejects non-finite and out-of-range values rather than
clamping; the getter rejects invalid native values. AVPlayer volume is relative to system volume and
affects only this player. The mute flag is also per-player. Neither control reports actual audible
output or changes system volume; this crate exposes no system-volume control.

The system reports eligible only when an HDR display is available and the device can play HDR
content from an appropriate `AVAsset`. The value does not say that a particular asset contains
HDR, that a current item can play, that playback is active, or that the display is currently
showing HDR. It is a point-in-time capability snapshot, not an observer. `None` means the call
was made off the main thread, on a non-iOS target, or below iOS 13.4; it must not be treated as
`Some(false)`.

The local-file API preflights absolute UTF-8 path input, rejects embedded NUL, opens the file, and
requires regular-file metadata before creating AVPlayer from the unchanged path. AVPlayer performs
asset loading and readiness checks asynchronously and owns codec support, buffering, decoding, and
output. The wrapper's play call is a request, not proof that playback or output began. The host owns
file lifetime, AVAudioSession configuration, permission policy, UI, and output-layer attachment.
The crate does not request permission or mutate AVAudioSession. The HDR query remains only a
system eligibility snapshot; it does not inspect or play an asset.

Apple APIs: [AVPlayer](https://developer.apple.com/documentation/avfoundation/avplayer),
[AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)

Run `sh platform/ios/ios-playback/check.sh` from the workspace root for formatting, iOS device and
simulator compile/Clippy, host compile/Clippy, documentation, and Release link/import checks. See
[local evidence](EVIDENCE.md). The link probes are build-only and are never executed. These gates do
not exercise HDR hardware, display routing, asset decoding, or runtime playback.
