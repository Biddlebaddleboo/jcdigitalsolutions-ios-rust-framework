#ifndef FRAMEWORK_C_API_H
#define FRAMEWORK_C_API_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkStatus;
typedef uint64_t FrameworkOperationHandle;
typedef uint64_t FrameworkErrorHandle;
typedef struct FrameworkErrorDetail FrameworkErrorDetail;
typedef FrameworkErrorDetail *FrameworkErrorDetailHandle;

#define FRAMEWORK_STATUS_OK UINT32_C(0)
#define FRAMEWORK_STATUS_INVALID_ARGUMENT UINT32_C(1)
#define FRAMEWORK_STATUS_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_STATUS_UNAVAILABLE UINT32_C(3)
#define FRAMEWORK_STATUS_PERMISSION_DENIED UINT32_C(4)
#define FRAMEWORK_STATUS_CANCELLED UINT32_C(5)
#define FRAMEWORK_STATUS_TIMEOUT UINT32_C(6)
#define FRAMEWORK_STATUS_NOT_FOUND UINT32_C(7)
#define FRAMEWORK_STATUS_ALREADY_EXISTS UINT32_C(8)
#define FRAMEWORK_STATUS_RESOURCE_EXHAUSTED UINT32_C(9)
#define FRAMEWORK_STATUS_PLATFORM_ERROR UINT32_C(10)
#define FRAMEWORK_STATUS_INTERNAL_ERROR UINT32_C(11)
#define FRAMEWORK_STATUS_PANIC UINT32_C(12)

typedef struct FrameworkSlice {
    const uint8_t *data;
    uint64_t length;
} FrameworkSlice;

typedef struct FrameworkStr {
    const uint8_t *data;
    uint64_t length;
} FrameworkStr;

typedef struct FrameworkOwnedBuffer {
    uint8_t *data;
    uint64_t length;
    uint64_t capacity;
} FrameworkOwnedBuffer;

typedef void (*FrameworkCompletionCallback)(
    void *context,
    FrameworkOperationHandle operation,
    FrameworkStatus status,
    FrameworkSlice result);

typedef struct FrameworkOptionsV1 {
    uint32_t struct_size;
    uint32_t abi_version;
    uint32_t flags;
    uint32_t reserved;
} FrameworkOptionsV1;

/* Major occupies bits 63..32; minor occupies bits 31..0. */
uint64_t framework_abi_version(void);

/*
 * Validates the V1 options prefix. `options` may be NULL, which returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without a read. Otherwise it must point to
 * valid, properly aligned, fully initialized readable FrameworkOptionsV1
 * storage for the full synchronous call. A size below sizeof(FrameworkOptionsV1)
 * or nonzero `reserved` returns FRAMEWORK_STATUS_INVALID_ARGUMENT. An `abi_version` other
 * than the major in framework_abi_version() returns FRAMEWORK_STATUS_UNSUPPORTED.
 * Same-major records of at least the V1 size are accepted. `flags` and any
 * trailing bytes are ignored. The pointer is not retained; callers must prevent
 * unsynchronized mutation during the call.
 */
FrameworkStatus framework_options_v1_validate(const FrameworkOptionsV1 *options);

/*
 * Copies a framework status code and a UTF-8 message into one directly owned, opaque error-detail
 * object. The supplied status is preserved exactly, including unknown future values; the function
 * return reports creation outcome and the stored code is read through framework_error_detail_view.
 * `out_detail` must
 * be non-NULL, writable, aligned, empty of a live object on entry, and disjoint from the message
 * bytes. It is reset to NULL before input validation and remains NULL on failure. A NULL message
 * pointer is valid only for zero length. Nonempty input must be readable and valid UTF-8 for this
 * synchronous call; bytes are copied exactly, embedded NUL is allowed, and no terminator is
 * added. Lengths that cannot fit a Rust slice or exceed isize's maximum are invalid.
 * Invalid non-NULL pointers violate the caller's FFI preconditions and are not detectable.
 * Allocation or capacity failure returns FRAMEWORK_STATUS_RESOURCE_EXHAUSTED. No input pointer
 * is retained. Valid preconditions follow a non-panicking path; no panic unwinds across C, and an
 * unexpected panic aborts at the extern C boundary.
 */
FrameworkStatus framework_error_detail_create(
    FrameworkStatus status,
    FrameworkStr message,
    FrameworkErrorDetailHandle *out_detail);

/*
 * Returns the view-call result and writes the stored status plus a borrowed message view. The view
 * remains valid until the detail object is destroyed. Both outputs must be non-NULL, writable, aligned, mutually
 * disjoint, and disjoint from the object/message storage; both are reset to OK and an empty string
 * before null-detail validation. A non-NULL `detail` must be the original aligned live handle
 * returned by create. If either output is NULL, the function returns
 * FRAMEWORK_STATUS_INVALID_ARGUMENT without writing either. The caller must keep `detail` live and
 * unchanged for the call, synchronize destruction against every borrowed view, and prevent
 * mutation through aliases.
 */
FrameworkStatus framework_error_detail_view(
    const FrameworkErrorDetail *detail,
    FrameworkStatus *out_status,
    FrameworkStr *out_message);

/* NULL is a no-op. Otherwise destroy the original live handle exactly once. Do not copy or
 * modify the handle, and do not use a borrowed message view after destruction. */
void framework_error_detail_destroy(FrameworkErrorDetailHandle detail);

/*
 * Copies bytes into a framework-owned allocation. `out_buffer` must be non-NULL, writable,
 * aligned, empty of a live allocation on entry, and disjoint from the input span. It is reset to
 * {NULL, 0, 0} before input validation and remains empty on failure. A nonzero input length
 * requires a non-NULL pointer to readable bytes for the full call; a NULL input pointer is valid
 * only when length is zero, and zero-length input succeeds without allocation. Lengths that
 * cannot fit a Rust slice return FRAMEWORK_STATUS_INVALID_ARGUMENT; allocation or descriptor
 * capacity failure returns FRAMEWORK_STATUS_RESOURCE_EXHAUSTED. The input is not retained. On
 * success, destroy the original unchanged output descriptor exactly once with
 * framework_owned_buffer_destroy. A NULL output returns FRAMEWORK_STATUS_INVALID_ARGUMENT
 * without a write.
 */
FrameworkStatus framework_owned_buffer_copy(FrameworkSlice bytes, FrameworkOwnedBuffer *out_buffer);

/* NULL is a no-op. Otherwise buffer must be an unchanged, live descriptor made by this framework.
 * Destroy the original descriptor once; never copy or mutate it, or destroy a copy. */
void framework_owned_buffer_destroy(FrameworkOwnedBuffer *buffer);

#ifdef __cplusplus
}
#endif

#endif
