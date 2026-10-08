#ifndef FRAMEWORK_NOTIFICATION_RESPONSES_H
#define FRAMEWORK_NOTIFICATION_RESPONSES_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkNotificationResponseKind;

#define FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DEFAULT UINT32_C(0)
#define FRAMEWORK_NOTIFICATION_RESPONSE_KIND_DISMISS UINT32_C(1)
#define FRAMEWORK_NOTIFICATION_RESPONSE_KIND_CUSTOM_ACTION UINT32_C(2)
#define FRAMEWORK_NOTIFICATION_RESPONSE_KIND_TEXT_INPUT UINT32_C(3)

typedef struct FrameworkNotificationResponse FrameworkNotificationResponse;

typedef struct FrameworkNotificationResponseViewV1 {
    FrameworkNotificationResponseKind kind;
    uint32_t reserved;
    FrameworkStr notification_id;
    FrameworkStr action_id;
    FrameworkStr user_text;
} FrameworkNotificationResponseViewV1;

/* Input spans must name valid UTF-8 and stay fixed through this call. Zero-length spans use
 * {NULL, 0}; notification/action IDs must be non-empty and contain no NUL byte. Text input may be
 * empty and may contain NUL bytes. Unused fields must be exactly {NULL, 0}. The call copies input
 * bytes before return. out_response must name storage with write access, distinct from inputs, and not hold a live
 * handle on entry; it is set to NULL before input checks */
FrameworkStatus framework_notification_response_create(
    FrameworkStr notification_id,
    FrameworkNotificationResponseKind kind,
    FrameworkStr action_id,
    FrameworkStr user_text,
    FrameworkNotificationResponse **out_response);

/* out_view must name storage with write access, distinct from response; it is set to zero before work
 * View spans borrow from response and stay valid only while the response stays alive. Do not race
 * get_view with destroy. Absent fields
 * use {NULL, 0}; kind marks empty text input */
FrameworkStatus framework_notification_response_get_view(
    const FrameworkNotificationResponse *response,
    FrameworkNotificationResponseViewV1 *out_view);

/* NULL is a no-op. For a live original handle, this clears its original slot before drop. Do not
 * race with get_view, copy the handle, destroy an alias, or use a view once this call returns */
void framework_notification_response_destroy(FrameworkNotificationResponse **response);

#ifdef __cplusplus
}
#endif

#endif
