# iOS VideoToolbox hardware-decode C API

The opt-in `ios-videotoolbox` feature exposes one status query over B50's current-system
`VTIsHardwareDecodeSupported` predicate. It reports hardware-decode support for one caller-supplied
FourCC only; it does not create decoder sessions, process frames, report software decoding, or
reserve hardware resources

## Call contract

`framework_ios_videotoolbox_hardware_decode_supported` takes a `uint32_t` containing the four
display-order FourCC bytes in big-endian order and one required writable output byte. For example,
`avc1` is `0x61766331`. The wrapper passes any 32-bit codec value to the system predicate without a
separate whitelist

When non-null, `out_supported` must address one valid, properly aligned writable byte for the full
synchronous call. The caller must prevent unsynchronized concurrent access. The API checks only nullness.
It does not retain the output pointer. The output is initialized to zero before platform
handling. On iOS 11.0 or later, `OK` writes exactly zero or one. If B50's runtime availability guard
reports the API unavailable, the wrapper returns `UNAVAILABLE` and zero output. A valid non-iOS
call returns `UNSUPPORTED` and zero output. A null output pointer returns
`FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write; a caught Rust panic returns `PANIC` with zero
output. No pointer or native object is retained or exposed

The public API is available from iOS 11.0. Device link probes use minos 11.0, matching the API
floor; Simulator probes use minos 14.0, which is only a link setting. A separate minos-10 link
experiment found a strong unresolved `_VTIsHardwareDecodeSupported` symbol, so F25 does not claim
weak-import safety or supported execution below iOS 11.0. No permission, Info.plist key, or
entitlement is required. The query is synchronous and adds no thread-affinity or thread-safety
promise

The F25 C/C++ link probes imported `VideoToolbox` and `libSystem.B.dylib` on device and Simulator;
CoreMedia and AVFAudio were not direct imports. The existing `ios-media` crate also compiles its
AVAudioSession dependency, but the final F25 links did not retain AVFAudio symbols. Host C linked
with `libSystem.B.dylib`; host C++ also linked `libc++.1.dylib`

`true` is only a point-in-time system report. It does not guarantee future decoder resources or a
successful decoder session. No tests or live hardware query are included in the link gate; linked
consumers and probes are inspected but never executed

See [F25 binding plan](../../PLAN_BINDINGS_F25.md), [B50 backend plan](../../PLAN_IOS_VIDEOTOOLBOX.md),
and [D45 capability contract](../../PLAN_CAPABILITIES_VIDEO_CODEC.md)
