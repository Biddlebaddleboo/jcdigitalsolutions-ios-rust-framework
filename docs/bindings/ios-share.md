# iOS share C ABI

F7 adds the opt-in `ios-share` feature to `framework-c-api`. It exposes the B7 `IosShareSession` over the F1 C ABI. The feature uses `ios-sharing/share` alone and does not enable `ios-sharing/clipboard`

## Scope

- Text and URL-text request items only
- One host-owned session with one active request at a time
- At most one host callback, only if UIKit reports a terminal share result
- UIKit context from the host app; no global registry or executor
- No promise of recipient delivery, visible presentation, or UI dismissal on cancel

F7 does not wrap D6 `ShareFuture`; that future borrows its backend. F7 uses the B7 callback session instead

## C API and tags

Include `framework_ios_share.h`. It includes `framework.h` and exposes these item tags

| Tag | Value | D6 type |
| --- | ---: | --- |
| `FRAMEWORK_IOS_SHARE_ITEM_TEXT` | 0 | `ShareItem::Text` |
| `FRAMEWORK_IOS_SHARE_ITEM_URL` | 1 | `ShareItem::Url` |

`FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED` is 0 and maps to `ShareOutcome::Completed`. `FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED` is 1 and maps to `ShareOutcome::Dismissed`. Neither tag means recipient delivery; `FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED` does not identify an actor or platform-specific cause

Availability tags use the fixed range 0 through 5. B7 reports `FRAMEWORK_IOS_SHARE_AVAILABILITY_UNKNOWN` and does not claim the UI context stays ready. A non-iOS stub sets `FRAMEWORK_IOS_SHARE_AVAILABILITY_UNSUPPORTED` and returns `FRAMEWORK_STATUS_UNSUPPORTED`

## Records and request rules

`FrameworkIosShareItemV1` has size 24 and alignment 8 on 64-bit iOS: `kind` at 0, `reserved` at 4, and `text` at 8

`FrameworkIosShareRequestV1` has size 24 and alignment 8: `struct_size` at 0, `abi_version` at 4, `items` at 8, and `item_count` at 16

`FrameworkIosShareAnchorV1` has size 40 and alignment 8: `struct_size` at 0, `abi_version` at 4, then `x`, `y`, `width`, and `height` at 8, 16, 24, and 32

- Request and anchor records require exact V1 size and ABI major 1
- Request requires at least one item; `items` must be non-null and `item_count` must fit `usize`
- Each item requires a known tag, zero `reserved`, and valid UTF-8
- Every empty text span is `{NULL, 0}`; embedded NUL is valid
- `start` copies all item text into owned D6 values before it returns; input spans do not outlive the call
- Item order stays as supplied
- URL text passes through D6; B7 may reject or normalize it for UIKit
- Anchor coordinates must be finite; width and height must be nonnegative
- Anchor coordinates use the `source_view` coordinate space

## Session and UIKit context

`framework_ios_share_session_create` requires live Objective-C objects of type `UIViewController` and `UIView`, passed as `void *`. Keep both objects live through create. On success, F7 and B7 retain them in the session. B7 checks that both views belong to the same window and validates the presentation context before UI start

Every non-null output slot and destroy handle slot must have natural alignment and writable storage. Keep output slots separate from input records and live handle storage

Create, availability, start, cancel, destroy, and the terminal callback must run on the iOS main thread. Serialize all calls for one session. Every status call checks the main thread before it reads a session handle

`framework_ios_share_session_destroy` returns `FRAMEWORK_STATUS_UNAVAILABLE` off-main without reading or changing the slot; retry on main. On main, pass the original writable slot once. Destroy clears that slot, cancels active callback state, then drops the session. A null slot or null handle returns `FRAMEWORK_STATUS_OK`

Do not copy a live handle, destroy an alias, race calls with destroy, or start a new request from inside its callback

## Callback, cancel, and status

`framework_ios_share_start` requires a non-null `FrameworkIosShareCompletion`. A preflight or input error returns its status from `start`; it does not call the callback and does not take callback or context ownership. A second start while a request is active returns `FRAMEWORK_STATUS_ALREADY_EXISTS`

`start` returns before callback; if UIKit reports a terminal result, callback runs once on main. UIKit may report no result, so accepted start does not guarantee a callback. The C host keeps `context` live until callback return, a successful cancel, or session destroy. The callback must not unwind into Rust or reenter F7. B7 detaches callback state before notify

- Completion maps to `FRAMEWORK_STATUS_OK`, `FRAMEWORK_IOS_SHARE_OUTCOME_COMPLETED`, and native code 0
- Dismissal maps to `FRAMEWORK_STATUS_OK`, `FRAMEWORK_IOS_SHARE_OUTCOME_DISMISSED`, and native code 0
- A known `ShareError::Backend` maps through `FrameworkStatus::from_error`; an unknown non-exhaustive error maps to `FRAMEWORK_STATUS_INTERNAL_ERROR`; outcome is 0
- Preserve a nonzero `NSError.code` only when it fits signed 32-bit range; otherwise native code is 0
- Unknown non-exhaustive D6 outcome or error variants map to `FRAMEWORK_STATUS_INTERNAL_ERROR`
- `framework_ios_share_cancel` returns `FRAMEWORK_STATUS_OK` only when it detaches an active callback; otherwise it returns `FRAMEWORK_STATUS_NOT_FOUND`

Cancel or destroy suppresses a late callback but does not promise to dismiss UIKit share UI already on screen. UIKit has no presentation-error callback, so a successful `start` only means that B7 requested presentation

## Non-iOS behavior and limits

Non-iOS builds expose explicit unsupported stubs. Valid operations initialize required outputs and validate input shapes, then return `FRAMEWORK_STATUS_UNSUPPORTED`; null-safe destroy returns `FRAMEWORK_STATUS_OK` for a null slot or null handle. They do not create a mock session or load UIKit

F7 adds no task executor, process-global map, delegate replacement, permission API, fabricated backend, delivery claim, or force-quit behavior. The C API exposes no UIKit class type

## Non-test checks

`bindings/c/check-ios-share.sh` checks feature isolation, host stubs, iOS device and simulator builds, strict Clippy, and C11/C++17 compile/link fixtures. It compares each linked C/C++ device and simulator probe with the exact direct-import set in `bindings/c/abi-manifest.json` via `otool -L`; `nm -u` checks each linked probe and archive for clipboard, Swift/Python, and unrelated framework symbols. It does not execute fixtures. Build checks do not prove live UI presentation or recipient behavior
