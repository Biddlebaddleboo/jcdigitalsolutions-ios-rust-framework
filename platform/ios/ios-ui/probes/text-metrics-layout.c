#include <CoreText/CTFont.h>

_Static_assert(sizeof(CGFloat) == 8, "CGFloat size must match the arm64 Rust alias");
_Static_assert(_Alignof(CGFloat) == 8, "CGFloat alignment must match the arm64 Rust alias");
_Static_assert(sizeof(CTFontUIFontType) == 4, "CTFontUIFontType must be uint32_t");

typedef CTFontRef (*CreateUIFontFn)(CTFontUIFontType, CGFloat, CFStringRef);
typedef CGFloat (*GetFontMetricFn)(CTFontRef);
typedef void (*ReleaseFontFn)(CFTypeRef);

_Static_assert(__builtin_types_compatible_p(__typeof__(&CTFontCreateUIFontForLanguage), CreateUIFontFn),
               "CTFontCreateUIFontForLanguage declaration changed");
_Static_assert(__builtin_types_compatible_p(__typeof__(&CTFontGetAscent), GetFontMetricFn),
               "CTFontGetAscent declaration changed");
_Static_assert(__builtin_types_compatible_p(__typeof__(&CTFontGetDescent), GetFontMetricFn),
               "CTFontGetDescent declaration changed");
_Static_assert(__builtin_types_compatible_p(__typeof__(&CTFontGetLeading), GetFontMetricFn),
               "CTFontGetLeading declaration changed");
_Static_assert(__builtin_types_compatible_p(__typeof__(&CFRelease), ReleaseFontFn),
               "CFRelease declaration changed");
