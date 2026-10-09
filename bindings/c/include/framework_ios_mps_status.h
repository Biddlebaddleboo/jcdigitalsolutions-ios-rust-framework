#ifndef FRAMEWORK_IOS_MPS_STATUS_H
#define FRAMEWORK_IOS_MPS_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-mps-status`. `MPSGetPreferredDevice(MPSDeviceOptionsDefault)` has an
 * iOS API floor of 12.2; no runtime availability check is performed, so call
 * only on iOS 12.2 or later. The linked Release probes use target minima 12.2
 * for device and 14.0 for Simulator; the Simulator minimum is a link setting,
 * not the API floor.
 *
 * A non-null `out_available` must address valid, properly aligned writable
 * memory for one byte during this synchronous call. The client must prevent
 * unsynchronized concurrent access.
 * The API checks nullness only; it does not retain the output address.
 * The wrapper sets it to zero before platform handling. A null pointer returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. On iOS, success writes
 * exactly zero or one according to whether MPS returns a preferred device
 * with default options. A valid non-iOS call returns
 * FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC with zero output.
 *
 * The retained native device is dropped before return and no handle crosses
 * C. This query submits no GPU work and does not establish support for any
 * operation, model, or workload. No permission, Info.plist key, or entitlement
 * is required. Runtime cost and thread affinity are unspecified; this API
 * adds no main-thread rule or thread-safety promise.
 */
FrameworkStatus framework_ios_mps_status_preferred_device_available(
    uint8_t *out_available);

#ifdef __cplusplus
}
#endif

#endif
