#include <stdbool.h>
#include <stddef.h>
#include <CoreGraphics/CGGeometry.h>

#if !defined(__LP64__)
#error "ios-ui geometry probe requires the current 64-bit iOS ABI"
#endif

_Static_assert(sizeof(bool) == 1, "C bool ABI");
_Static_assert(sizeof(CGFloat) == 8, "CGFloat size");
_Static_assert(_Alignof(CGFloat) == 8, "CGFloat alignment");
_Static_assert(sizeof(CGPoint) == 16, "CGPoint size");
_Static_assert(_Alignof(CGPoint) == 8, "CGPoint alignment");
_Static_assert(offsetof(CGPoint, x) == 0, "CGPoint.x offset");
_Static_assert(offsetof(CGPoint, y) == 8, "CGPoint.y offset");
_Static_assert(sizeof(CGSize) == 16, "CGSize size");
_Static_assert(_Alignof(CGSize) == 8, "CGSize alignment");
_Static_assert(offsetof(CGSize, width) == 0, "CGSize.width offset");
_Static_assert(offsetof(CGSize, height) == 8, "CGSize.height offset");
_Static_assert(sizeof(CGRect) == 32, "CGRect size");
_Static_assert(_Alignof(CGRect) == 8, "CGRect alignment");
_Static_assert(offsetof(CGRect, origin) == 0, "CGRect.origin offset");
_Static_assert(offsetof(CGRect, size) == 16, "CGRect.size offset");

int ios_ui_geometry_layout_probe(void) {
    return 0;
}
