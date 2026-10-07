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

/* NULL is a no-op. Otherwise buffer must be an unchanged, live descriptor made by this framework.
 * Destroy the original descriptor once; never copy or mutate it, or destroy a copy. */
void framework_owned_buffer_destroy(FrameworkOwnedBuffer *buffer);

#ifdef __cplusplus
}
#endif

#endif
