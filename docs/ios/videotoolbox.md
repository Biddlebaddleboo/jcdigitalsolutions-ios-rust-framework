# VideoToolbox hardware-decode support

`framework-media` defines a portable FourCC codec value and a scalar hardware-decode support
result. `ios-media` implements `hardware_decode_support(codec)` with Apple's
`VTIsHardwareDecodeSupported` predicate.

Enable the `ios-media` Cargo feature `videotoolbox` to use this adapter. It is disabled by default,
so callers using only CoreMedia time or audio-session status do not link VideoToolbox.

```rust
use framework_media::VideoCodecType;

let h264 = VideoCodecType::from_fourcc(*b"avc1");
match ios_media::hardware_decode_support(h264) {
    Some(support) if support.is_supported() => { /* hardware decode is reported */ }
    Some(_) => { /* hardware decode is not reported for this codec */ }
    None => { /* non-iOS or runtime earlier than iOS 11.0 */ }
}
```

The API floor is iOS 11.0. `None` means the API is unavailable on this target/runtime. `Some(false)`
does not rule out software decode. Apple also notes that a positive result does not guarantee
hardware decoder resources will remain available for a later operation.

This is a capability snapshot only. It does not report encoder support, create a decoder session,
load or inspect media assets, process frames, reserve resources, or request permission. It exposes
no VideoToolbox handle and makes no live-device, codec-quality, or performance claim.

`VideoCodecType::from_fourcc` takes the four bytes in display order: H.264 is `*b"avc1"`; HEVC is
`*b"hvc1"`. The portable contract does not expose CoreMedia or VideoToolbox types.

The backend uses typed `objc2-video-toolbox` 0.3.2 bindings with default features disabled. The
package gate is `sh platform/ios/ios-media/check.sh`; it checks portable, host, device, and simulator
builds, strict Clippy, and docs, but runs no tests or codec operation.

Apple references: [`VTIsHardwareDecodeSupported`](https://developer.apple.com/documentation/videotoolbox/vtishardwaredecodesupported%28_%3A%29), [`CMVideoCodecType`](https://developer.apple.com/documentation/coremedia/cmvideocodectype).
