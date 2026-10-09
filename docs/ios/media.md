# iOS CoreMedia Value Bridge

`ios-media::IosMediaTime` maps `framework_media::MediaTime` to a CoreMedia `CMTime` value

```rust
use framework_media::MediaTime;
use ios_media::IosMediaTime;

let time = MediaTime::new(1001, 30_000)?;
let native = IosMediaTime::from_portable(time);
let cm_time = native.as_cm_time();
```

The API calls `CMTime::new` with the same numerator and positive timescale. CoreMedia sets epoch zero. The accessor returns the native struct by value; there is no object owner, raw pointer, float conversion, or scale change

The inspected iOS SDK 26.5 marks `CMTime` and `CMTimeMake` as available from iOS 4.0. The link probe uses iOS 12.0 for device and iOS 14.0 for Simulator; those probe targets do not raise the API floor. No permission, entitlement, or Info.plist key is needed

`sh platform/ios/ios-media/check-link-imports.sh` links device and Simulator probes with `-Wl,-dead_strip_dylibs`, checks the C header and Rust `CMTime` layout, then inspects direct imports of CoreMedia and libSystem plus deployment metadata. It does not run either probe. The CoreFoundation load command is absent because no imported symbol requires it; `_CMTimeMake` remains imported from CoreMedia. The separate `ios_media::other_audio_playback_snapshot()` AVFAudio query is documented in the [other-audio guide](other-audio.md); these CoreMedia checks do not prove live audio state, media playback, capture, parity, or performance
