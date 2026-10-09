#ifndef FRAMEWORK_IOS_TRANSFER_H
#define FRAMEWORK_IOS_TRANSFER_H

#include <framework.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FrameworkIosTransferClient FrameworkIosTransferClient;
typedef struct FrameworkIosTransferSnapshot FrameworkIosTransferSnapshot;

typedef uint32_t FrameworkTransferAvailability;
#define FRAMEWORK_TRANSFER_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_TRANSFER_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_TRANSFER_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_TRANSFER_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_TRANSFER_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef uint32_t FrameworkTransferDirectory;
#define FRAMEWORK_TRANSFER_DIRECTORY_DOCUMENTS UINT32_C(0)
#define FRAMEWORK_TRANSFER_DIRECTORY_CACHES UINT32_C(1)
#define FRAMEWORK_TRANSFER_DIRECTORY_TEMPORARY UINT32_C(2)
#define FRAMEWORK_TRANSFER_DIRECTORY_APPLICATION_SUPPORT UINT32_C(3)

typedef uint32_t FrameworkTransferState;
#define FRAMEWORK_TRANSFER_STATE_QUEUED UINT32_C(0)
#define FRAMEWORK_TRANSFER_STATE_ACTIVE UINT32_C(1)
#define FRAMEWORK_TRANSFER_STATE_SUCCEEDED UINT32_C(2)
#define FRAMEWORK_TRANSFER_STATE_FAILED UINT32_C(3)
#define FRAMEWORK_TRANSFER_STATE_CANCELLED UINT32_C(4)

typedef uint32_t FrameworkTransferErrorKind;
#define FRAMEWORK_TRANSFER_ERROR_KIND_UNKNOWN UINT32_C(0)
#define FRAMEWORK_TRANSFER_ERROR_KIND_INVALID_INPUT UINT32_C(1)
#define FRAMEWORK_TRANSFER_ERROR_KIND_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_TRANSFER_ERROR_KIND_UNAVAILABLE UINT32_C(3)
#define FRAMEWORK_TRANSFER_ERROR_KIND_PERMISSION_DENIED UINT32_C(4)
#define FRAMEWORK_TRANSFER_ERROR_KIND_CANCELLED UINT32_C(5)
#define FRAMEWORK_TRANSFER_ERROR_KIND_TIMEOUT UINT32_C(6)
#define FRAMEWORK_TRANSFER_ERROR_KIND_NOT_FOUND UINT32_C(7)
#define FRAMEWORK_TRANSFER_ERROR_KIND_ALREADY_EXISTS UINT32_C(8)
#define FRAMEWORK_TRANSFER_ERROR_KIND_RESOURCE_EXHAUSTED UINT32_C(9)
#define FRAMEWORK_TRANSFER_ERROR_KIND_PLATFORM UINT32_C(10)
#define FRAMEWORK_TRANSFER_ERROR_KIND_INTERNAL UINT32_C(11)

typedef struct FrameworkTransferIdV1 {
    uint64_t high;
    uint64_t low;
} FrameworkTransferIdV1;

typedef struct FrameworkTransferHeaderV1 {
    FrameworkStr name;
    FrameworkSlice value;
} FrameworkTransferHeaderV1;

typedef struct FrameworkTransferRequestV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkTransferIdV1 id;
    FrameworkStr url;
    const FrameworkTransferHeaderV1 *headers;
    uint64_t header_count;
    FrameworkTransferDirectory directory;
    uint32_t reserved;
    FrameworkStr relative_path;
} FrameworkTransferRequestV1;

typedef struct FrameworkTransferSnapshotViewV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    FrameworkTransferIdV1 id;
    FrameworkTransferState state;
    FrameworkTransferErrorKind failure_kind;
    int32_t native_code;
    uint32_t http_status;
    uint32_t reserved;
    uint32_t reserved2;
    uint64_t response_header_count;
} FrameworkTransferSnapshotViewV1;

typedef struct FrameworkTransferHeaderViewV1 {
    FrameworkStr name;
    FrameworkSlice value;
} FrameworkTransferHeaderViewV1;

typedef void (*FrameworkIosTransferEventsCompletion)(void *context);

/* Non-NULL output pointers must meet their C type's alignment and permit writes. Keep outputs
 * distinct from each other, input spans, request records, header arrays, and live handle storage */

/* Client creation and all client calls must run on the main thread
 * The app owns UIApplicationDelegate and forwards the matching session ID to this handle
 * Synchronous task, journal, and file calls may block the main thread
 * A non-NULL out_client is set to NULL before validation and must not hold a live client */
FrameworkStatus framework_ios_transfer_client_create(
    FrameworkStr session_identifier,
    FrameworkIosTransferClient **out_client,
    int32_t *out_native_code);

/* NULL is a no-op; otherwise pass the original slot once, on the main thread, after all accepted
 * event callbacks run. On iOS, an off-main client-destroy call is a no-op and leaves the slot
 * unchanged. The function clears the slot before drop. Do not copy or alias a handle */
void framework_ios_transfer_client_destroy(FrameworkIosTransferClient **client);

FrameworkStatus framework_ios_transfer_availability(
    const FrameworkIosTransferClient *client,
    FrameworkTransferAvailability *out_availability);

/* The request record and non-empty header array must meet their C type's alignment. Request spans
 * are borrowed through the call; start_download copies accepted data before return. Every zero-
 * length span must be {NULL, 0}; headers is NULL iff header_count is zero */
FrameworkStatus framework_ios_transfer_start_download(
    FrameworkIosTransferClient *client,
    const FrameworkTransferRequestV1 *request,
    int32_t *out_native_code);

/* out_snapshot must not hold a live snapshot; each non-NULL required output is set to an empty
 * value before work */
FrameworkStatus framework_ios_transfer_status(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    FrameworkIosTransferSnapshot **out_snapshot,
    uint8_t *out_found,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_cancel(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    int32_t *out_native_code);

FrameworkStatus framework_ios_transfer_forget(
    FrameworkIosTransferClient *client,
    FrameworkTransferIdV1 id,
    int32_t *out_native_code);

/* The output record is fully overwritten; its fields are output metadata, not input validation */
FrameworkStatus framework_ios_transfer_snapshot_get_view(
    const FrameworkIosTransferSnapshot *snapshot,
    FrameworkTransferSnapshotViewV1 *out_view);

/* out_header is set to empty spans before work. Header spans borrow from the snapshot until
 * snapshot_destroy */
FrameworkStatus framework_ios_transfer_snapshot_get_header(
    const FrameworkIosTransferSnapshot *snapshot,
    uint64_t index,
    FrameworkTransferHeaderViewV1 *out_header);

/* NULL is a no-op; otherwise pass the original slot once. This clears the slot before drop */
void framework_ios_transfer_snapshot_destroy(FrameworkIosTransferSnapshot **snapshot);

/* Forward UIKit's callback to the exact live client for session_identifier
 * completion must be non-NULL
 * A successful call takes callback/context ownership until one callback on the main dispatch queue
 * B13 calls it once after the URLSession finish-events callback and initial task reconciliation
 * It cannot run before this function returns. Keep the client and context alive until that callback
 * A rejected call does not take ownership; callback must not unwind into Rust */
FrameworkStatus framework_ios_transfer_forward_background_events(
    FrameworkIosTransferClient *client,
    FrameworkStr session_identifier,
    FrameworkIosTransferEventsCompletion completion,
    void *context,
    int32_t *out_native_code);

/* Call once after UIKit routing rules out a background-session event for this launch
 * Do not also call forward_background_events for that launch */
FrameworkStatus framework_ios_transfer_finish_launch_without_background_events(
    FrameworkIosTransferClient *client,
    int32_t *out_native_code);

#ifdef __cplusplus
}
#endif

#endif
