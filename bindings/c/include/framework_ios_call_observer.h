#ifndef FRAMEWORK_IOS_CALL_OBSERVER_H
#define FRAMEWORK_IOS_CALL_OBSERVER_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkIosCallObserverStateFlags;
#define FRAMEWORK_IOS_CALL_OBSERVER_STATE_OUTGOING UINT32_C(1)
#define FRAMEWORK_IOS_CALL_OBSERVER_STATE_CONNECTED UINT32_C(2)
#define FRAMEWORK_IOS_CALL_OBSERVER_STATE_ON_HOLD UINT32_C(4)
#define FRAMEWORK_IOS_CALL_OBSERVER_STATE_ENDED UINT32_C(8)

/*
 * This header is opt-in through the framework-c-api Cargo feature
 * `ios-call-observer`. On iOS it synchronously reads CXCallObserver.calls and
 * returns only that list's count plus a bitwise union of four state facts.
 * The read may block while CallKit retrieves initial state; do not call on
 * UI-critical work. There is no main-thread requirement.
 *
 * Each non-null output is initialized to zero before work. Both outputs are
 * required, distinct, writable, and aligned. Read them only on
 * FRAMEWORK_STATUS_OK. A null required output returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT. Non-iOS stubs return
 * FRAMEWORK_STATUS_UNSUPPORTED with both outputs zero. A caught Rust panic
 * returns FRAMEWORK_STATUS_PANIC with both outputs zero.
 *
 * No CallKit object, UUID, caller data, callback, history, or call-control
 * API is exposed. The snapshot is point-in-time and may be stale on return.
 */
FrameworkStatus framework_ios_call_observer_active_call_snapshot(
    uint64_t *out_call_count,
    FrameworkIosCallObserverStateFlags *out_state_flags);

#ifdef __cplusplus
}
#endif

#endif
