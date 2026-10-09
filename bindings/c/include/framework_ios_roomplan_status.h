#ifndef FRAMEWORK_IOS_ROOMPLAN_STATUS_H
#define FRAMEWORK_IOS_ROOMPLAN_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Opt in with framework-c-api's `ios-roomplan-status` Cargo feature. This exposes only B61's
 * synchronous `RoomCaptureSession.isSupported` result. The API and required iOS deployment floor
 * are 16.0. The B61 backend and F31 C/C++ link/import probes use minos 16.0 for device and
 * Simulator. F31 links directly to RoomPlan and libSystem without Swift or Objective-C runtime
 * imports.
 *
 * `out_supported` must address valid, properly aligned writable memory for one byte
 * for the full synchronous call. The API checks only nullness. The caller must
 * prevent unsynchronized concurrent access to the byte. The output is initialized to zero
 * before platform handling and the pointer is not retained. A null pointer returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. On iOS, success returns FRAMEWORK_STATUS_OK
 * and writes exactly zero or one according to B61's RoomPlan support result. A valid non-iOS call
 * returns FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC with zero output.
 *
 * This does not create or run a capture session, access camera or LiDAR frames, request permission,
 * present UI, begin a scan, or establish that a scan can start or succeed. It adds no thread or
 * queue guarantee.
 */
FrameworkStatus framework_ios_roomplan_status_is_supported(uint8_t *out_supported);

#ifdef __cplusplus
}
#endif

#endif
