#ifndef FRAMEWORK_IOS_VIDEOTOOLBOX_H
#define FRAMEWORK_IOS_VIDEOTOOLBOX_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-videotoolbox`. `codec_fourcc` is the FourCharCode as a big-endian
 * uint32_t with its four bytes in display order; for example, `avc1` is
 * 0x61766331. Any uint32_t is passed to the system predicate without a
 * separate codec whitelist.
 *
 * The VideoToolbox query has an iOS API floor of 11.0. On iOS 11.0 or later,
 * success writes exactly zero or one according to
 * `VTIsHardwareDecodeSupported(codec_fourcc)`. Below iOS 11.0, the wrapper
 * returns FRAMEWORK_STATUS_UNAVAILABLE and leaves the output zero. A valid
 * non-iOS call returns FRAMEWORK_STATUS_UNSUPPORTED and leaves the output
 * zero. The linked Release probes use minos 11.0 for device, matching the API
 * floor, and 14.0 for Simulator. The Simulator minimum is a link setting,
 * not the API floor; F25 makes no pre-iOS-11 weak-import claim.
 *
 * When non-null, `out_supported` must address one valid, properly aligned
 * writable byte for the full synchronous call. The caller must prevent
 * unsynchronized concurrent access. The API checks only nullness and does not
 * retain the output pointer. The output is initialized to zero before platform
 * handling. A null output pointer returns FRAMEWORK_STATUS_INVALID_ARGUMENT
 * without a write. A caught Rust panic returns FRAMEWORK_STATUS_PANIC with zero output.
 * No permission, Info.plist key, or entitlement is required, and
 * this query creates no decoder session or processes media data. A true
 * result does not reserve decoder resources or guarantee a later session.
 * This API adds no thread-affinity or thread-safety promise.
 */
FrameworkStatus framework_ios_videotoolbox_hardware_decode_supported(
    uint32_t codec_fourcc,
    uint8_t *out_supported);

#ifdef __cplusplus
}
#endif

#endif
