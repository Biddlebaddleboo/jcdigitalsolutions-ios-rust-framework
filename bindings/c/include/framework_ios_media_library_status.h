#ifndef FRAMEWORK_IOS_MEDIA_LIBRARY_STATUS_H
#define FRAMEWORK_IOS_MEDIA_LIBRARY_STATUS_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef int64_t FrameworkIosMediaLibraryAuthorizationStatus;
#define FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_NOT_DETERMINED INT64_C(0)
#define FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_DENIED INT64_C(1)
#define FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_RESTRICTED INT64_C(2)
#define FRAMEWORK_IOS_MEDIA_LIBRARY_AUTHORIZATION_AUTHORIZED INT64_C(3)

/*
 * This header is opt-in through the framework-c-api Cargo feature
 * `ios-media-library-status`. On iOS, the query calls
 * +[MPMediaLibrary authorizationStatus] and is available from iOS 9.3. Use a
 * compatible deployment target or an explicit runtime availability guard.
 *
 * Known raw statuses are 0 NOT_DETERMINED, 1 DENIED, 2 RESTRICTED, and 3
 * AUTHORIZED. Other signed values are returned unchanged for forward
 * compatibility. The query does not request permission, show a prompt, create
 * a media-library object, read media items, or contact Apple Music services.
 * It reports authorization state only, not catalog, subscription, or playback
 * availability.
 *
 * out_raw_status is required and is initialized to zero before the query. Read
 * it only when FRAMEWORK_STATUS_OK is returned. On non-iOS targets the
 * function returns FRAMEWORK_STATUS_UNSUPPORTED and leaves the output zero.
 * The API makes no thread-safety promise and adds no main-thread rule.
 */
FrameworkStatus framework_ios_media_library_authorization_status(
    FrameworkIosMediaLibraryAuthorizationStatus *out_raw_status);

#ifdef __cplusplus
}
#endif

#endif
