# Portable video codec values

`framework-media::VideoCodecType` stores a four-byte FourCC in display order. For example, H.264
uses `*b"avc1"` and HEVC uses `*b"hvc1"`. Construction is platform-independent and does not imply
that a codec is supported on any device.

`HardwareDecodeSupport` is a copied Boolean snapshot from a platform query. A positive value does
not reserve decoder resources or guarantee a later decode operation. The portable crate does not
encode video, access media assets, create sessions, or process frames.

The iOS backend asks only VideoToolbox's `VTIsHardwareDecodeSupported` predicate through
[`ios-media`](../ios/videotoolbox.md), available from iOS 11.0. Its value is support metadata, not a
promise of future decoder availability.
