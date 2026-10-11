# Playback capability

`framework-audio` defines `HdrPlaybackEligibility`, a portable, allocation-free snapshot of one
backend-reported boolean. It does not expose a portable player, media item, asset, decoder, or
playback controls. The iOS extension also offers a main-thread local-file `AVPlayer` wrapper with
play/pause and a borrowed native-player escape hatch.

The snapshot answers only whether the selected backend reports system HDR playback eligibility at
the time of the query. It does not identify an HDR asset, current item, active playback, or HDR
output on the display. See the [iOS playback API](../ios/playback.md) for AVPlayer path, readiness,
session-policy, and runtime limits.
