# Playback snapshot status

Status: implemented as a bounded status-only query; no asset, player instance, or playback action.

Capability: `AVPlayer.eligibleForHDRPlayback` reports whether an HDR display is available and the
device can play HDR content from an appropriate asset. It says nothing about a current item or
active HDR output. `None` means non-iOS, off-main-thread, or below the iOS API floor; `Some(false)`
is only the system's negative snapshot.

iOS: the property is available from iOS 13.4. `objc2::available!(ios = 13.4, ..)` gates the call;
the portable value is in `framework-audio`; the typed binding is `objc2-av-foundation` 0.3.2 with
only the `AVPlayer` feature. No prompt,
protected-resource access, or playback session is used.

Validation: `sh platform/ios/ios-playback/check.sh` passed on Xcode 26.6 (build 17F113), iPhoneOS
SDK 26.5. It runs format checks, locked device and simulator `cargo check`, strict library Clippy
for both iOS targets, host check/Clippy, `cargo doc --no-deps`, and the build-only Release link/import
probe. `git diff --check` passed.

The device probe sets `IPHONEOS_DEPLOYMENT_TARGET=13.4` and confirms Mach-O `minos 13.4`; the
simulator probe sets `IPHONEOS_DEPLOYMENT_TARGET=14.0` and confirms `minos 14.0` (the Rust simulator
target floor). Both exceed the API's iOS 13.4 floor. Exact expected imports for both probes are
`AVFoundation`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`. The probe confirms the
`eligibleForHDRPlayback` selector and `_objc_msgSend`, with no Swift runtime, audio, capture, asset,
or player-item imports. Neither binary is executed.

No tests or live device/HDR playback checks were run; hardware and runtime behavior remain
unverified.
