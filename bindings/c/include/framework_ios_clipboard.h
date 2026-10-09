#ifndef FRAMEWORK_IOS_CLIPBOARD_H
#define FRAMEWORK_IOS_CLIPBOARD_H

#include "framework.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t FrameworkIosClipboardAvailability;

#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN UINT32_C(0)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE UINT32_C(1)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED UINT32_C(2)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION UINT32_C(3)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT UINT32_C(4)
#define FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE UINT32_C(5)

typedef struct FrameworkIosClipboard FrameworkIosClipboard;

/* Every non-null output slot must be naturally aligned and writable
 * Keep output storage separate from input spans, handles, and other outputs
 */
/* Create and use a clipboard client only on the iOS main thread
 * out_clipboard is set to NULL before checks; it must not name a live handle on entry
 */
FrameworkStatus framework_ios_clipboard_create(
    FrameworkIosClipboard **out_clipboard);

/* On iOS, destroy runs on main and clears the original handle slot before drop
 * The handle slot must be naturally aligned and writable
 * Off-main destroy is a no-op; a null slot or null handle is safe
 */
void framework_ios_clipboard_destroy(
    FrameworkIosClipboard **clipboard);

/* Writes UNKNOWN before checks on iOS and UNSUPPORTED on non-iOS
 * UIKit exposes no query for current programmatic-read usability or approval
 * UNKNOWN does not mean that permission is required
 */
FrameworkStatus framework_ios_clipboard_availability(
    const FrameworkIosClipboard *clipboard,
    FrameworkIosClipboardAvailability *out_availability);

/* None => has_value 0 and empty buffer; Some("") => has_value 1 and empty buffer
 * B6 reads the first string returned by UIPasteboard.strings
 * Required outputs must be non-null and are set to empty before checks
 * Each non-null output must be disjoint from other outputs, the handle, and input storage
 * If has_value is 1, release out_text once with framework_owned_buffer_destroy, even at length 0
 * If has_value is 0, out_text is the default empty descriptor
 * out_text must be empty on entry. Optional native-code output may be NULL; a non-null value is zeroed before checks
 * Current B6 errors provide no native code
 */
FrameworkStatus framework_ios_clipboard_read(
    FrameworkIosClipboard *clipboard,
    uint8_t *out_has_value,
    FrameworkOwnedBuffer *out_text,
    int32_t *out_native_code);

/* Writes exact UTF-8 through UIPasteboard.string, replacing all current pasteboard items
 * This removes any existing non-text representations; text is borrowed only through the call
 * Empty text is {NULL, 0}, and embedded NUL is valid
 * Optional native-code output may be NULL; a non-null value is set to zero before checks
 */
FrameworkStatus framework_ios_clipboard_write(
    FrameworkIosClipboard *clipboard,
    FrameworkStr text,
    int32_t *out_native_code);

/* B6 assigns an empty UIPasteboard.items array, removing all current items including non-text ones
 * Optional native-code output may be NULL; a non-null value is set to zero before checks
 */
FrameworkStatus framework_ios_clipboard_clear(
    FrameworkIosClipboard *clipboard,
    int32_t *out_native_code);

#ifdef __cplusplus
}
#endif

#endif
