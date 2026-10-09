#include <Photos/PHPhotoLibrary.h>

#if __IPHONE_OS_VERSION_MIN_REQUIRED < 140000
#error "PhotoKit read/write authorization requires iOS 14.0 or later"
#endif

_Static_assert(PHAccessLevelReadWrite == 2, "unexpected read/write access level value");
_Static_assert(PHAuthorizationStatusNotDetermined == 0, "unexpected not-determined status value");
_Static_assert(PHAuthorizationStatusRestricted == 1, "unexpected restricted status value");
_Static_assert(PHAuthorizationStatusDenied == 2, "unexpected denied status value");
_Static_assert(PHAuthorizationStatusAuthorized == 3, "unexpected authorized status value");
_Static_assert(PHAuthorizationStatusLimited == 4, "unexpected limited status value");
