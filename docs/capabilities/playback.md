# Playback capability

`framework-audio` defines `HdrPlaybackEligibility`, a portable, allocation-free snapshot of one
backend-reported boolean. It does not expose a portable player, media item, asset, decoder, or
playback controls. The iOS extension also offers a main-thread local-file `AVPlayer` wrapper with
play/pause, exact-current-item end observation, an asynchronous seek-completion future, and a
borrowed native-player escape hatch.

The snapshot answers only whether the selected backend reports system HDR playback eligibility at
the time of the query. It does not identify an HDR asset, current item, active playback, or HDR
output on the display. The iOS extension also offers validated per-player relative volume and mute
controls; these do not access device/system volume or establish audible output. See the
[iOS playback API](../ios/playback.md) for item-observation lifetime, AVPlayer path, readiness,
session-policy, and runtime limits.
