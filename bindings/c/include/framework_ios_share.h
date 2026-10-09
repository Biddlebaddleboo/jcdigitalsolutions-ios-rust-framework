#ifndef FRAMEWORK_IOS_SHARE_H
#define FRAMEWORK_IOS_SHARE_H

#include <framework.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosShareSession FrameworkIosShareSession;

typedef uint32_t FrameworkIosShareItem;
#define FRAMEWORK_IOS_SHARE_ITEM_TEXT UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_ITEM_URL UINT32_C(1)

typedef uint32_t FrameworkIosShareOutcome;
#define FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED UINT32_C(1)

typedef uint32_t FrameworkIosShareAvailability;
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_SHARE_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef struct FrameworkIosShareItemV1 {
    uint32_t kind;
    uint32_t reserved;
    FrameworkStr text;
} FrameworkIosShareItemV1;

typedef struct FrameworkIosShareRequestV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    const FrameworkIosShareItemV1 *items;
    uint64_t item_count;
} FrameworkIosShareRequestV1;

typedef struct FrameworkIosShareAnchorV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    double x;
    double y;
    double width;
    double height;
} FrameworkIosShareAnchorV1;

typedef void (*FrameworkIosShareCompletion)(
    void *context,
    FrameworkStatus status,
    FrameworkIosShareOutcome outcome,
    int32_t native_code);

/* Every non-null output slot and destroy handle slot must be naturally aligned and writable */
/* Create and use one session on the iOS main thread
 * out_session is initialized to NULL before validation; keep it separate from input records and live handle storage
 * Pass live UIViewController and UIView objects as the context pair
 * The session retains both objects after create
 */
FrameworkStatus framework_ios_share_session_create(
    void *presenter,
    void *source_view,
    FrameworkIosShareAnchorV1 anchor,
    FrameworkIosShareSession **out_session);

/* Availability is UNKNOWN on iOS and UNSUPPORTED on non-iOS targets
 * out_availability is initialized to UNKNOWN before validation and must not alias the live handle
 */
FrameworkStatus framework_ios_share_session_availability(
    const FrameworkIosShareSession *session,
    FrameworkIosShareAvailability *out_availability);

/* Request spans are borrowed through this call and copied before accepted start
 * Every empty span is {NULL, 0}; item order stays as supplied
 * No callback runs before start returns
 * If UIKit reports a terminal result, the callback runs once; accepted start does not guarantee a callback
 * Keep callback and context live until callback return, successful cancel, or destroy
 * A rejected start does not call the callback or take callback/context ownership
 * Callback must be non-null, must not unwind, and must not reenter this C API
 */
FrameworkStatus framework_ios_share_start(
    FrameworkIosShareSession *session,
    const FrameworkIosShareRequestV1 *request,
    FrameworkIosShareCompletion completion,
    void *context);

/* Detach an active result callback; this does not dismiss visible UIKit share UI */
FrameworkStatus framework_ios_share_cancel(
    FrameworkIosShareSession *session);

/* The original handle slot must be naturally aligned and writable
 * Off-main calls return UNAVAILABLE without reading or changing the slot
 * On main, pass the original slot once; destroy cancels active callback state
 */
FrameworkStatus framework_ios_share_session_destroy(
    FrameworkIosShareSession **session);

#ifdef __cplusplus
}
#endif

#endif
