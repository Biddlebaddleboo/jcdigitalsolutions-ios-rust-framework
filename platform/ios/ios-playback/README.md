# iOS playback capability snapshot

`ios-playback` exposes one narrow query: `hdr_playback_eligibility()` reads
`AVPlayer.eligibleForHDRPlayback` on the main thread. The property is available from iOS 13.4;
the query returns `None` below that version. It links AVFoundation through the typed
`objc2-av-foundation` 0.3.2 `AVPlayer` feature.

The system reports eligible only when an HDR display is available and the device can play HDR
content from an appropriate `AVAsset`. The value does not say that a particular asset contains
HDR, that a current item can play, that playback is active, or that the display is currently
showing HDR. It is a point-in-time capability snapshot, not an observer. `None` means the call
was made off the main thread, on a non-iOS target, or below iOS 13.4; it must not be treated as
`Some(false)`.

The query does not create a player, inspect or load media, start playback, request permission, or
perform network or device I/O. It makes no claim about general audio/video playback availability.
No entitlement or privacy usage string is required for this capability query.

Apple API: [AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)

Run `sh platform/ios/ios-playback/check.sh` from the workspace root for formatting, iOS device and
simulator compile/Clippy, host compile/Clippy, documentation, and Release link/import checks. See
[local evidence](EVIDENCE.md). The link probes are build-only and are never executed. These gates do
not exercise HDR hardware, display routing, asset decoding, or runtime playback.
