#ifndef FRAMEWORK_IOS_LOCATION_H
#define FRAMEWORK_IOS_LOCATION_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosLocationOperation FrameworkIosLocationOperation;
typedef uint32_t FrameworkIosLocationAvailability;
typedef uint32_t FrameworkIosLocationAuthorization;
typedef uint32_t FrameworkIosLocationOperationKind;

#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_LOCATION_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_NOT_DETERMINED UINT32_C(1)
#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_DENIED UINT32_C(2)
#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_RESTRICTED UINT32_C(3)
#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_FOREGROUND UINT32_C(4)
#define FRAMEWORK_IOS_LOCATION_AUTHORIZATION_BACKGROUND UINT32_C(5)

#define FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_QUERY UINT32_C(0)
#define FRAMEWORK_IOS_LOCATION_OPERATION_AUTHORIZATION_REQUEST UINT32_C(1)
#define FRAMEWORK_IOS_LOCATION_OPERATION_CURRENT UINT32_C(2)

typedef void (*FrameworkIosLocationReady)(void *context);

typedef struct FrameworkIosLocationResultV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkIosLocationOperationKind operation_kind;
    FrameworkStatus status;
    FrameworkIosLocationAuthorization authorization;
    uint32_t reserved;
    double latitude_degrees;
    double longitude_degrees;
    double horizontal_accuracy_meters;
    uint64_t timestamp_unix_millis;
    int32_t native_code;
    uint32_t reserved2;
} FrameworkIosLocationResultV1;

/*
 * This header is opt-in through the framework-c-api Cargo feature `ios-location`.
 * The API wraps only B5's Core Location availability, authorization query,
 * explicit foreground-authorization request, and one-shot current request.
 * It creates no continuous or background location service and does not expose
 * an Objective-C object.
 *
 * The Core Location API floor is iOS 9.0 because requestLocation() was
 * introduced in iOS 9.0. The installed Rust arm64 iOS target builds objects
 * with minos 10.0, so the focused device link gate uses iOS 10.0; the focused
 * Simulator link gate uses iOS 14.0. This does not establish full-library
 * runtime support on iOS 9.0. A consuming app that requests
 * authorization must provide NSLocationWhenInUseUsageDescription. Only the
 * explicit authorization_request_start function can request foreground
 * permission; it may display Apple's permission UI, can remain pending if no
 * authorization change is reported, and never requests Always or background
 * access. A current-location request does not request permission by itself.
 *
 * Pointer preconditions are caller obligations: every output pointer must name
 * valid storage with the required natural alignment and writable extent for the
 * full call. Availability output must not overlap a live iOS operation handle.
 * A start's output handle slot must not contain a live handle on entry or
 * overlap live operation storage. Poll requires one live unique iOS handle and
 * two aligned writable outputs that do not overlap each other or the handle.
 * Cancel requires that same unique live
 * handle. Destroy accepts NULL or the original writable handle slot returned
 * by start; do not copy or alias a handle, or race any operation with destroy.
 *
 * Every iOS call, poll, cancel, and destroy must run on the main thread. The
 * app's main run loop must continue to run. Each accepted start owns one
 * capability-scoped operation handle before any readiness callback can run;
 * a successful start returns FRAMEWORK_STATUS_OK. There is no global registry
 * or shared executor. The readiness callback is only a one-shot
 * signal to poll, not a terminal result callback. It runs on the main thread
 * and may run before a start function returns for an immediately-ready query
 * or result. Do not unwind or re-enter this ABI from the callback; schedule a
 * later main-thread poll after it returns. Poll also works without waiting for
 * the callback. Keep callback context alive until the callback returns, a
 * successful cancel, a ready poll result, or destroy. The framework does not
 * own or free callback context.
 *
 * Poll initializes out_ready to 0 and the complete output record before
 * validation. Inspect result fields only when the call returns
 * FRAMEWORK_STATUS_OK and out_ready is 1. A ready result is consumed once;
 * poll after consumption returns FRAMEWORK_STATUS_NOT_FOUND. A successful
 * cancel drops the pending future on main and suppresses later callback/native
 * result delivery; a later poll returns a ready FRAMEWORK_STATUS_CANCELLED result.
 * Cancel after readiness notification or terminal result returns
 * FRAMEWORK_STATUS_NOT_FOUND. Poll to consume a terminal result if it has not
 * already been consumed; poll after consumption also returns
 * FRAMEWORK_STATUS_NOT_FOUND. Destroy clears the original handle slot before
 * drop. Off-main destroy returns
 * FRAMEWORK_STATUS_UNAVAILABLE without reading or changing the slot.
 * Destroying or cancelling a pending authorization request abandons its
 * result but cannot promise to dismiss permission UI already shown. Dropping a
 * pending current-location request asks Core Location to stop that operation;
 * no GPS, freshness, accuracy-target, or time-to-fix guarantee is made.
 *
 * FrameworkIosLocationResultV1 is output-only, 64 bytes with 8-byte alignment
 * on 64-bit iOS. It has struct_size at 0, abi_version at 4, operation_kind at
 * 8, status at 12, authorization at 16, reserved at 20, latitude_degrees at
 * 24, longitude_degrees at 32, horizontal_accuracy_meters at 40,
 * timestamp_unix_millis at 48, native_code at 56, and reserved2 at 60.
 *
 * Non-iOS builds validate required pointers and input values, initialize valid
 * outputs, then return FRAMEWORK_STATUS_UNSUPPORTED. They create no mock
 * handle and import no Apple framework; a non-null unsupported destroy slot
 * remains unchanged. A caught Rust panic returns
 * FRAMEWORK_STATUS_PANIC; no panic unwinds across C.
 */
FrameworkStatus framework_ios_location_availability(
    FrameworkIosLocationAvailability *out_availability);

FrameworkStatus framework_ios_location_authorization_query_start(
    FrameworkIosLocationReady completion,
    void *context,
    FrameworkIosLocationOperation **out_operation);

FrameworkStatus framework_ios_location_authorization_request_start(
    FrameworkIosLocationReady completion,
    void *context,
    FrameworkIosLocationOperation **out_operation);

FrameworkStatus framework_ios_location_current_start(
    double accuracy_target_meters,
    FrameworkIosLocationReady completion,
    void *context,
    FrameworkIosLocationOperation **out_operation);

FrameworkStatus framework_ios_location_operation_poll(
    FrameworkIosLocationOperation *operation,
    uint8_t *out_ready,
    FrameworkIosLocationResultV1 *out_result);

FrameworkStatus framework_ios_location_operation_cancel(
    FrameworkIosLocationOperation *operation);

FrameworkStatus framework_ios_location_operation_destroy(
    FrameworkIosLocationOperation **operation);

#ifdef __cplusplus
}
#endif

#endif
