#ifndef FRAMEWORK_IOS_CAMERA_DEVICE_STATUS_H
#define FRAMEWORK_IOS_CAMERA_DEVICE_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature
 * `ios-camera-device-status`. It exposes only B62's synchronous,
 * point-in-time `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`
 * query. The public API floor is iOS 4.0. F26's Release link probes use
 * minos 10.0 for device and 14.0 for Simulator; these are probe deployment
 * settings, not the API floor.
 *
 * `out_present` is required and writable for one byte. The API checks only
 * nullness: any non-null pointer must actually address valid, properly
 * aligned writable memory for the synchronous call. The caller must prevent
 * unsynchronized concurrent access to that byte. The pointer is initialized
 * to zero before platform handling and is not retained. A null pointer returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a write. On iOS, success returns
 * FRAMEWORK_STATUS_OK and writes exactly zero or one according to whether
 * AVFoundation returns a current default video device. A valid non-iOS call
 * returns FRAMEWORK_STATUS_UNSUPPORTED with zero output. A caught Rust panic
 * returns FRAMEWORK_STATUS_PANIC with zero output.
 *
 * This query does not request camera authorization, create a device input or
 * capture session, read media, or present UI. Device presence is not
 * authorization, capture readiness, configured session state, or future
 * availability. A host app still needs `NSCameraUsageDescription` before it
 * requests camera access or creates an input. This API adds no thread or
 * queue guarantee.
 */
FrameworkStatus framework_ios_camera_device_status_has_default_video_capture_device(
    uint8_t *out_present);

#ifdef __cplusplus
}
#endif

#endif
