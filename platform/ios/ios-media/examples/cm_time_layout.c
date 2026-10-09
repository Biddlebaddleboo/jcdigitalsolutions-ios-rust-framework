#include <stddef.h>
#include <CoreMedia/CMTime.h>

_Static_assert(sizeof(CMTime) == 24, "unexpected CMTime size");
_Static_assert(_Alignof(CMTime) == 4, "unexpected CMTime alignment");
_Static_assert(offsetof(CMTime, value) == 0, "unexpected CMTime.value offset");
_Static_assert(offsetof(CMTime, timescale) == 8, "unexpected CMTime.timescale offset");
_Static_assert(offsetof(CMTime, flags) == 12, "unexpected CMTime.flags offset");
_Static_assert(offsetof(CMTime, epoch) == 16, "unexpected CMTime.epoch offset");
