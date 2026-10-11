# PLAN_IOS_PLAYBACK.md — Workstream B48: HDR Playback Eligibility Query

## Status

B48 adds a safe Rust query over `AVPlayer.eligibleForHDRPlayback`. The public API floor is iOS
13.4. The runtime guard returns `None` below that floor, off the main thread, and on non-iOS
targets. The typed `objc2-av-foundation` 0.3.2 binding is enabled only for `AVPlayer`. Device,
Simulator, host, Clippy, rustdoc, and Release import gates pass locally on Xcode 26.6 build 17F113 /
iOS SDK 26.5. The package gate is wired in macOS CI; no tests or live HDR checks were run, and no
passing CI workflow run is recorded.

## Objective

Expose one status-only AVFoundation query through Rust while leaving player construction, asset
loading, decoding, playback, display routing, and user interface to the host application and
Apple frameworks.

## Dependencies

- D43 `framework-audio::HdrPlaybackEligibility`
- Public `AVPlayer.eligibleForHDRPlayback`, available from iOS 13.4
- `objc2-av-foundation` 0.3.2 with only the `AVPlayer` feature

## API and limits

- `hdr_playback_eligibility()` returns `None` off the main thread, below iOS 13.4, or on non-iOS
  targets; otherwise it returns the system boolean snapshot.
- Require an `objc2::MainThreadMarker` and guard the class property with
  `objc2::available!(ios = 13.4, ..)`.
- A positive value concerns HDR display availability and device capability for appropriate media;
  it does not prove a particular asset or current item is HDR or that HDR output is active.
- Do not create an `AVPlayer`, inspect or load assets, start playback, request permission, or add a
  Swift source file.
- No live device, HDR display, asset decode, route-change, runtime parity, or performance claim.

## Validation

Run `sh platform/ios/ios-playback/check.sh`. It checks formatting, device and Simulator builds,
strict Clippy for both iOS targets, host check/Clippy, and rustdoc. It does not run tests or exercise
HDR hardware/runtime behavior.

Apple API: [AVPlayer.eligibleForHDRPlayback](https://developer.apple.com/documentation/avfoundation/avplayer/eligibleforhdrplayback)

## B463 — local-file player controls (row 048)

B463 extends this crate with a thin main-thread `IosFilePlayer` around the public `AVPlayer` URL
constructor. It accepts an absolute Rust UTF-8 path, rejects embedded NUL, opens the file to verify
readability and regular-file metadata, then passes the unchanged path to `NSURL::fileURLWithPath`.
It provides main-thread `play` and `pause` calls plus a borrowed `&AVPlayer` escape for host
`AVPlayerLayer` integration.

The iPhoneOS SDK declares `AVPlayer` from iOS 4.0; local creation, play, and pause therefore use
that API floor, and row 048 records the aggregate AVPlayer playback floor as iOS 4.0. The
pre-iOS-16 main-thread requirement is enforced by `MainThreadMarker` on create, controls, and
native escape. The existing HDR property remains guarded at iOS 13.4. Validator target floors stay
at iOS 13.4 for device and iOS 14.0 for Simulator.

No seek/current-time methods were added by B463: portable `framework-media::MediaTime` represents finite
numeric times only, while `AVPlayer` time can be invalid or indefinite. AVPlayer handles codec
support, readiness, buffering, decoding, and output asynchronously. The host owns the file's
post-construction lifetime, `AVAudioSession` policy, permission policy, UI, and output attachment.
The local-file API makes no claim that `play()` began playback or that audio/video output works.

## B466 — asynchronous seek completion (row 048)

B466 adds `IosFilePlayer::seek_to(MediaTime, MainThreadMarker)` with an eagerly started AVPlayer
seek and a nonblocking `IosPlayerSeekFuture`. `MediaTime` maps to a `CMTime` struct with the exact
signed numerator, positive timescale, valid flag, and epoch zero. The future borrows its player and
retains the main-thread marker. Concurrent seeks are allowed; AVPlayer may report an earlier seek
as `NotCompleted` when superseded. A false completion flag also means there may be no current item,
so the result does not identify the cause. Drop detaches Rust result/waker state without requesting
native cancellation. Completion proves only AVPlayer's uninterrupted seek flag, not decoding,
display, active playback, or output. No current-time/duration query is included.

The public `AVPlayer.seekToTime:completionHandler:` binding requires a sendable block. The block
captures only an owned `Arc` completion cell, which settles once on any callback thread and catches
panics before they cross Objective-C. The Rust future remains non-`Send`, borrows `IosFilePlayer`,
and is therefore valid only while the caller retains the player and main-thread context. The direct
`objc2-core-media` dependency enables only `CMTime`; `ios-media` is not used because it also enables
an unrelated AVAudioSession binding. The existing iOS 13.4 device and iOS 14.0 Simulator validation
floors remain unchanged; the public AVPlayer seek API floor remains iOS 4.0.

Host tests cover exact rational fields, both completion results, single settlement, wake behavior,
detachment, late callback, and callback/waker panic containment. Device/Simulator compile and
Release link checks prove the typed ABI shape, direct CoreMedia/AVFoundation imports, selector string,
and deployment metadata. They do not prove a seek on an item, decoding, displayed output, or live
playback. Apple API: [AVPlayer.seekToTime:completionHandler:](https://developer.apple.com/documentation/avfoundation/avplayer/seek%28to%3Acompletionhandler%3A%29).

## B492 — per-player AVPlayer volume and mute controls

B492 adds `IosFilePlayer::volume`, `set_volume`, `is_muted`, and `set_muted`. All four methods require `objc2::MainThreadMarker`, use generated typed AVPlayer property calls, and return `IosPlayerOutputError::NativeApiUnavailable` below iOS 7.0. The existing iOS 4.0 aggregate API floor and registered device/Simulator floors stay unchanged.

`set_volume` accepts only finite values in `[0.0, 1.0]`; it rejects invalid input without a native call or clamp. The getter returns only finite in-range native values. AVPlayer volume is a per-player level relative to system volume and is not suitable for a user-facing system-volume slider. `is_muted` and `set_muted` expose only the player mute flag, not device mute state, route suppression, or audible output. No system-volume API or AVAudioSession mutation is used.

The iPhoneOS 26.5 SDK header declares both properties from iOS 7.0, and generated `objc2-av-foundation` 0.3.2 exposes typed `volume`, `setVolume:`, `isMuted`, and `setMuted:` methods under the existing `AVPlayer` feature. Host regressions cover inclusive endpoints, non-finite and out-of-range values, and invalid native reads. The Apple-target profile remains pending; no device/Simulator compile, selector/link, runtime, or audible-output claim is made.

Apple APIs: [AVPlayer.volume](https://developer.apple.com/documentation/avfoundation/avplayer/volume) and [AVPlayer.isMuted](https://developer.apple.com/documentation/avfoundation/avplayer/ismuted).
