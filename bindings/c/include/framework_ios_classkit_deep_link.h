#ifndef FRAMEWORK_IOS_CLASSKIT_DEEP_LINK_H
#define FRAMEWORK_IOS_CLASSKIT_DEEP_LINK_H

#include "framework.h"

#ifdef __OBJC__
@class NSUserActivity;
#else
typedef struct NSUserActivity NSUserActivity;
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef uint8_t FrameworkIosClassKitBoolean;

/*
 * This header is opt-in through the framework-c-api Cargo feature
 * `ios-classkit-deep-link`. Pass a live, caller-owned NSUserActivity. The
 * wrapper borrows the activity only for this synchronous call; it does not
 * retain, store, or release the pointer. Keep the activity alive and use it
 * only within the host app's activity lifecycle and thread requirements.
 *
 * The function reads only `isClassKitDeepLink`. It does not read the context
 * identifier path, ClassKit contexts, assignment data, or user identity. This
 * marker is not access to ClassKit assignment data.
 *
 * `out_is_deep_link` is required, must not overlap the activity object, is
 * initialized to 0 before input or runtime checks, and is set to exactly 0 or
 * 1 only on FRAMEWORK_STATUS_OK. A null
 * activity or output returns FRAMEWORK_STATUS_INVALID_ARGUMENT. The getter's
 * runtime/API floor is iOS 11.3; an earlier iOS runtime returns
 * FRAMEWORK_STATUS_UNAVAILABLE with output 0. A non-null pointer on non-iOS
 * returns FRAMEWORK_STATUS_UNSUPPORTED without dereference and leaves output
 * 0. A caught Rust panic returns FRAMEWORK_STATUS_PANIC with output 0.
 */
FrameworkStatus framework_ios_classkit_is_deep_link(
    const NSUserActivity *activity,
    FrameworkIosClassKitBoolean *out_is_deep_link);

#ifdef __cplusplus
}
#endif

#endif
