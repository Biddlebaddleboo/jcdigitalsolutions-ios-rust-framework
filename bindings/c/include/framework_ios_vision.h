#ifndef FRAMEWORK_IOS_VISION_H
#define FRAMEWORK_IOS_VISION_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

/*
 * This header is opt-in through framework-c-api's Cargo feature `ios-vision`.
 * It queries whether the current iOS runtime lists `revision` in
 * +[VNRecognizeTextRequest supportedRevisions], available from iOS 13.0.
 * This is revision membership only: no request, handler, image, user data,
 * permission request, UI, recognition, or model-readiness query is made.
 *
 * `out_supported` must address valid, properly aligned writable uint8_t memory
 * for the full synchronous call. The caller must prevent unsynchronized access
 * to the output. The wrapper does not retain the output address. The output is
 * initialized to zero before the query. On FRAMEWORK_STATUS_OK, it is exactly
 * zero or one. Any uint32_t
 * revision is valid; an unknown revision returns OK with zero. A runtime
 * below iOS 13.0 returns FRAMEWORK_STATUS_UNAVAILABLE with zero. Non-iOS
 * targets return FRAMEWORK_STATUS_UNSUPPORTED with zero. A null output
 * returns FRAMEWORK_STATUS_INVALID_ARGUMENT. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC. The pointer is borrowed for this synchronous call
 * and is not retained. The API adds no main-thread rule or thread-safety
 * guarantee.
 */
FrameworkStatus framework_ios_vision_text_recognition_revision_is_supported(
    uint32_t revision,
    uint8_t *out_supported);

#ifdef __cplusplus
}
#endif

#endif
