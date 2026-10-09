# iOS HDR playback eligibility

`ios-playback::hdr_playback_eligibility()` reads the public
`AVPlayer.eligibleForHDRPlayback` class property through the typed `objc2-av-foundation` 0.3.2
binding. The call requires iOS 13.4 or later and the main thread. It returns `None` off-main-thread,
below iOS 13.4, or on a non-iOS target.

The result is Apple's point-in-time HDR eligibility status: the system reports whether an HDR
display is available and the device can play HDR content from an appropriate asset. It does not
inspect a particular asset, create or query a player instance, start playback, confirm HDR output,
or report general media availability. No permission prompt or Swift source is involved.

The local package gate checks formatting, host/device/Simulator compilation and Clippy, and
rustdoc. No test, link-import probe, physical HDR display, live asset, or runtime playback behavior
is claimed.

Apple API: [AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)
