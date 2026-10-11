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

No seek/current-time methods are added: portable `framework-media::MediaTime` represents finite
numeric times only, while `AVPlayer` time can be invalid or indefinite. AVPlayer handles codec
support, readiness, buffering, decoding, and output asynchronously. The host owns the file's
post-construction lifetime, `AVAudioSession` policy, permission policy, UI, and output attachment.
The local-file API makes no claim that `play()` began playback or that audio/video output works.
