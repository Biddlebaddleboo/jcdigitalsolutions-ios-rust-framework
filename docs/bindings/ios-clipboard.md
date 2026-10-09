# iOS clipboard C ABI

The opt-in `ios-clipboard` feature exposes D5 plain-text read, write, and clear operations through
`framework_ios_clipboard.h`. It uses B6's `IosClipboardBackend`; it adds no rich pasteboard values,
share UI, global state, or executor.

## Build and target behavior

The default `framework-c-api` feature set contains no clipboard symbols or clipboard backend. Add
the `ios-clipboard` Cargo feature only for an iOS device or simulator library. The C feature selects
`ios-sharing/clipboard` with default features disabled, so it does not enable B7's share module,
`block2`, or `UIActivityViewController`. The `ios-sharing` crate itself keeps both `clipboard` and
`share` enabled by default for existing Rust consumers.

On non-iOS targets, the same symbols initialize required outputs and do not create a fake
clipboard. Valid calls return `FRAMEWORK_STATUS_UNSUPPORTED`; required output pointers are checked
first, and `write` validates the `FrameworkStr` pointer/length shape and UTF-8 before the host stub
returns. Invalid pointers or malformed text can therefore return
`FRAMEWORK_STATUS_INVALID_ARGUMENT` instead. The availability output is
`FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED`.

## Threading and operations

Create, availability, read, write, clear, and destroy must run on the iOS main thread. Calls on
another thread return `FRAMEWORK_STATUS_UNAVAILABLE` for otherwise-valid status calls; malformed
inputs may return `FRAMEWORK_STATUS_INVALID_ARGUMENT` first. Destroy has no status and is a no-op
off-main. The C caller must serialize access to each handle and must not race an operation with
destroy.

Create returns one unique opaque `FrameworkIosClipboard` handle. Destroy clears the original handle
slot before releasing its backend; do not copy the handle or destroy an alias. Availability is
`Unknown` because UIKit exposes no query for current programmatic-read usability or approval. This
does not mean that permission is required.

D5 operations return Rust futures, but B6 performs its UIKit work synchronously on first poll. Each C
read, write, and clear call polls once on the main thread; an unexpected `Pending` maps to
`FRAMEWORK_STATUS_INTERNAL_ERROR`. No C executor, callback, operation table, or asynchronous API is
added.

`FrameworkStr` input is exact UTF-8 borrowed for the call. Empty input is `{NULL, 0}`; embedded NUL
bytes are valid. B6 copies write input into an `NSString`, and UIKit copies the assigned value; no C
input span is retained after the call. Assigning `UIPasteboard.string` replaces all current
pasteboard items, including any non-text representations. A read copies the first string
from `UIPasteboard.strings`
into an owned `String` and then an owned `FrameworkOwnedBuffer`, so no readable text (`None`) differs
from a present empty string (`Some("")`). When `out_has_value` is 1, release `out_text` exactly once
with `framework_owned_buffer_destroy`, even when its length is zero; when it is 0, the descriptor is
the default empty value. The output descriptor must be empty on entry. Other apps may
change shared pasteboard state at any time; read, write, or clear does not reserve or guarantee a
later value

Every non-null output slot and destroy handle slot must be naturally aligned and writable. Keep
output storage separate from input spans, handles, and other outputs. Create requires an output slot
that does not name a live handle. Read requires non-null `out_has_value` and `out_text`, sets all
available outputs to empty before it checks those pointers, and requires `out_text` to be empty on
entry. Its optional native-code output may be null, and every non-null native-code output starts at
zero

Known `ClipboardError::Backend` values map through the portable `ErrorKind` to `FrameworkStatus`;
unknown non-exhaustive errors map to `FRAMEWORK_STATUS_INTERNAL_ERROR`. The optional native-code
output is initialized to zero and remains zero for current B6 errors. B6's `clear` implementation
assigns an empty `UIPasteboard.items` array, removing all current items including non-text
representations.

## Privacy and limits

The B6 guide, [iOS clipboard and sharing](../ios/sharing.md), documents Apple's OS-dependent
privacy behavior: iOS 14 and later notify users when an app reads general-pasteboard content from
another app without user intent, and iOS 16 and later may show an approval alert for programmatic
reads. The `UIPasteControl` user-mediated path is out of scope, and this C ABI exposes no permission
state. These are documented platform semantics, not live prompt or pasteboard verification in this
workstream; confirm behavior in the host app on each target OS. The API exposes only the general
pasteboard's plain-text contract, not arbitrary pasteboard items, files, images, rich text, or the
share sheet. The D6 share contract has no C surface in this slice; a later share C ABI needs a
separate safe completion/session design.
