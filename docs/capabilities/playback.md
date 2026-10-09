# Playback capability

`framework-audio` defines `HdrPlaybackEligibility`, a portable, allocation-free snapshot of one
backend-reported boolean. It does not expose an audio engine, player, media item, asset, decoder, or
playback controls.

The snapshot answers only whether the selected backend reports system HDR playback eligibility at
the time of the query. It does not identify an HDR asset, current item, active playback, or HDR
output on the display. See the [iOS query](../ios/playback.md) for the AVFoundation scope and
runtime limits.
