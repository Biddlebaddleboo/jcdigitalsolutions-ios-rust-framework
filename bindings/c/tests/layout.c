#include "framework.h"

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#define ROUND_UP(value, alignment) (((value) + (alignment) - 1) / (alignment) * (alignment))
#define AGGREGATE_ALIGN ((uint64_t)(_Alignof(uint64_t) > _Alignof(void *) ? _Alignof(uint64_t) : _Alignof(void *)))
#define LENGTH_OFFSET ROUND_UP(sizeof(void *), _Alignof(uint64_t))

_Static_assert(offsetof(FrameworkSlice, data) == 0, "slice data offset");
_Static_assert(offsetof(FrameworkSlice, length) == LENGTH_OFFSET, "slice length offset");
_Static_assert(sizeof(FrameworkSlice) == ROUND_UP(LENGTH_OFFSET + sizeof(uint64_t), AGGREGATE_ALIGN), "slice size");
_Static_assert(_Alignof(FrameworkSlice) == AGGREGATE_ALIGN, "slice align");
_Static_assert(offsetof(FrameworkStr, data) == 0, "string data offset");
_Static_assert(offsetof(FrameworkStr, length) == LENGTH_OFFSET, "string length offset");
_Static_assert(sizeof(FrameworkStr) == sizeof(FrameworkSlice), "string size");
_Static_assert(_Alignof(FrameworkStr) == _Alignof(FrameworkSlice), "string align");
_Static_assert(offsetof(FrameworkOwnedBuffer, data) == 0, "buffer data offset");
_Static_assert(offsetof(FrameworkOwnedBuffer, length) == LENGTH_OFFSET, "buffer length offset");
_Static_assert(offsetof(FrameworkOwnedBuffer, capacity) == LENGTH_OFFSET + sizeof(uint64_t), "buffer capacity offset");
_Static_assert(sizeof(FrameworkOwnedBuffer) == ROUND_UP(LENGTH_OFFSET + 2 * sizeof(uint64_t), AGGREGATE_ALIGN), "buffer size");
_Static_assert(_Alignof(FrameworkOwnedBuffer) == AGGREGATE_ALIGN, "buffer align");

int main(void) {
    printf("{\"FrameworkStatus\":{\"size\":%zu,\"align\":%zu},", sizeof(FrameworkStatus), _Alignof(FrameworkStatus));
    printf("\"FrameworkSlice\":{\"size\":%zu,\"align\":%zu,\"fields\":{\"data\":%zu,\"length\":%zu}},", sizeof(FrameworkSlice), _Alignof(FrameworkSlice), offsetof(FrameworkSlice, data), offsetof(FrameworkSlice, length));
    printf("\"FrameworkStr\":{\"size\":%zu,\"align\":%zu,\"fields\":{\"data\":%zu,\"length\":%zu}},", sizeof(FrameworkStr), _Alignof(FrameworkStr), offsetof(FrameworkStr, data), offsetof(FrameworkStr, length));
    printf("\"FrameworkOwnedBuffer\":{\"size\":%zu,\"align\":%zu,\"fields\":{\"data\":%zu,\"length\":%zu,\"capacity\":%zu}},", sizeof(FrameworkOwnedBuffer), _Alignof(FrameworkOwnedBuffer), offsetof(FrameworkOwnedBuffer, data), offsetof(FrameworkOwnedBuffer, length), offsetof(FrameworkOwnedBuffer, capacity));
    printf("\"FrameworkOperationHandle\":{\"size\":%zu,\"align\":%zu},", sizeof(FrameworkOperationHandle), _Alignof(FrameworkOperationHandle));
    printf("\"FrameworkErrorHandle\":{\"size\":%zu,\"align\":%zu},", sizeof(FrameworkErrorHandle), _Alignof(FrameworkErrorHandle));
    printf("\"FrameworkCompletionCallback\":{\"size\":%zu,\"align\":%zu},", sizeof(FrameworkCompletionCallback), _Alignof(FrameworkCompletionCallback));
    printf("\"FrameworkOptionsV1\":{\"size\":%zu,\"align\":%zu,\"fields\":{\"struct_size\":%zu,\"abi_version\":%zu,\"flags\":%zu,\"reserved\":%zu}}}\n", sizeof(FrameworkOptionsV1), _Alignof(FrameworkOptionsV1), offsetof(FrameworkOptionsV1, struct_size), offsetof(FrameworkOptionsV1, abi_version), offsetof(FrameworkOptionsV1, flags), offsetof(FrameworkOptionsV1, reserved));
    return 0;
}
