# iOS sharing

This guide covers the `ios-sharing` crate's plain-text clipboard backend and outgoing system-share
backend. The clipboard section below retains its existing behavior and privacy notes; the share
backend is a separate UIKit presentation path.

## Outgoing system share

`IosShareBackend` implements `framework_sharing::ShareBackend` with UIKit's public
`UIActivityViewController`. It supports only owned UTF-8 text and URL-text items. It does not
support files, images, rich representations, custom activities, recipient data, activity IDs,
previews, extensions, clipboard operations, or a delivery guarantee.

### UI context and use

The caller supplies a live presenting `UIViewController`, a source `UIView`, and a `CGRect` in the
source view's coordinate system, along with `MainThread::current()`'s typed proof. Keep the
presenter and source view alive for the backend's lifetime, and construct, poll, and drop the
backend and its futures on the main thread. `IosShareBackend`, its callback state, and its futures
are `!Send`.

The share operation starts on its first future poll. It builds a non-empty activity-item array in
request order and supplies no custom `UIActivity` values. On iPad the controller uses popover
presentation with the caller's explicit `sourceView` and `sourceRect`; on phone it is presented
modally. UIKit defines the source rectangle in the source view's coordinate space. The source view
and the loaded presenter view must be attached to the same window.

~~~rust
use framework_sharing::{ShareClient, ShareError, ShareItem, ShareOutcome, ShareRequest};
use framework_core::{Error, ErrorKind};
use ios_runtime::main_thread::MainThread;
use ios_sharing::IosShareBackend;
use objc2_core_foundation::CGRect;
use objc2_ui_kit::{UIView, UIViewController};

async fn share_text<'ctx>(
    presenter: &'ctx UIViewController,
    source_view: &'ctx UIView,
    source_rect: CGRect,
) -> Result<ShareOutcome, ShareError> {
    let main_thread = MainThread::current()
        .ok_or(ShareError::Backend(Error::new(ErrorKind::Unavailable)))?;
    let backend = IosShareBackend::new(main_thread, presenter, source_view, source_rect);
    let mut client = ShareClient::new(backend);
    client
        .share(ShareRequest::new(ShareItem::text(String::from("Hello"))))
        .await
}
~~~

`availability()` returns `Availability::Unknown`: it does not claim that a presenter remains
visible or ready. Before presentation, the backend returns `ErrorKind::Unavailable` when the
presenter is transitioning, already presents another controller, has no loaded view/window, or
the source view has no window. A source view in a different window returns `ErrorKind::InvalidInput`.
These checks provide a bounded error for known invalid context; UIKit's presentation method itself
has no error callback.

### Payload, results, and cancellation

Text items become owned `NSString` objects. URL text follows Foundation's public
`NSURL.URLWithString:` path; a nil parse result and file URLs return `ErrorKind::InvalidInput`.
Accepted URLs may be canonicalized or percent-encoded according to the linked OS, so exact URL
string preservation is not promised. The owned native items are passed to the activity controller
in the portable request's order.

UIKit's completion callback maps `completed == true` without an error to `ShareOutcome::Completed`,
`completed == false` without an error to `ShareOutcome::Dismissed`, and a non-null `NSError` to
`ShareError::Backend` with `ErrorKind::Platform`. A non-zero error code that fits in `i32` is
preserved as the optional platform code. A result does not prove that a recipient received or kept
the shared content.

The callback is exactly-once from Rust's perspective, and unwinding callback panics are contained
before control returns to UIKit. With `panic=abort`, a panic aborts the process instead of
unwinding. Dropping the future before its first poll starts no native work. Dropping it after
presentation clears the waker/result state and removes the UIKit completion handler, so late native
callbacks are inert. The system UI may remain visible; dropping is not a dismissal or rollback
guarantee.

The backend adds no permission prompt, app-specific usage string, entitlement, executor, global
registry, or Swift source. No `Info.plist` key or entitlement was identified for constructing and
presenting `UIActivityViewController`; the selected destination activity may have separate behavior
or host-app requirements, which this backend does not configure.

### API floor and evidence limits

The effective API floor is iOS 8.0: `UIActivityViewController` itself is older, but its
`completionWithItemsHandler` and the popover anchor APIs used here are available from iOS 8.0.
`UIViewController.present` predates that floor. The active host has Xcode 26.6 (build 17F113),
iPhoneOS SDK 26.5, and iPhoneSimulator SDK 26.5; this remains below the repository's planned Xcode
27.x baseline. The local SDK headers and Apple's public references were checked for the APIs and
availability floor. No device-only availability restriction was found in the reviewed public
UIKit declarations. Device and simulator consumers both link, but no simulator or device UI action
was run.

Apple references: [`UIActivityViewController`](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller),
[`UIActivityViewController` initializer](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/init%28activityitems%3Aapplicationactivities%3A%29),
[`completionWithItemsHandler`](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/completionwithitemshandler-swift.property),
[`UIViewController.present`](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29),
[`UIPopoverPresentationController.sourceView`](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourceview),
[`UIPopoverPresentationController.sourceRect`](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourcerect),
and [`NSURL.URLWithString:`](https://developer.apple.com/documentation/foundation/nsurl/urlwithstring%3A).

Host fake-backend checks, target checks, and link/import inspection are not live share-sheet evidence.
They do not establish simulator/device UI behavior, recipient delivery, parity, or performance.

## Plain-text clipboard

`ios-sharing` implements the `framework-sharing::ClipboardBackend` contract with UIKit's systemwide general `UIPasteboard`. It supports plain-text read, write, and clear only. It does not include a share sheet, `UIPasteControl`, named pasteboards, or rich representations.

Outgoing text and URL-text requests use the separate portable [share contract](../capabilities/share.md); UIKit presentation is a distinct backend workstream.

## Create and use the backend

~~~rust
use framework_sharing::Clipboard;
use ios_runtime::main_thread::MainThread;
use ios_sharing::IosClipboardBackend;

fn clipboard() -> Option<Clipboard<IosClipboardBackend>> {
    let main_thread = MainThread::current()?;
    Some(Clipboard::new(IosClipboardBackend::new(main_thread)))
}
~~~

Construct, poll, and drop the backend and its futures on the main thread. The backend and all operation futures are `!Send`; the typed `MainThread` proof and non-send markers prevent safe Rust from moving them to a worker thread. Each operation performs its synchronous UIKit access on its first future poll and returns `Ready` in that poll; dropping an unpolled future has no pasteboard effect. There is no pending native operation to cancel, and a completed write or clear cannot be undone by dropping its future. The backend adds no queue, executor, Objective-C callback, task registry, or hidden initialization.

## Reads, writes, clear, and copies

Read first checks `UIPasteboard.hasStrings` and only then loads `UIPasteboard.strings`. The backend copies the first string value in that array, including when an earlier pasteboard item has no string representation. No string type returns `Ok(None)`. If another app changes the shared pasteboard between the preflight and value read, or Foundation cannot encode the native string as UTF-8, the backend returns `ClipboardError::Backend` rather than reporting no value. Foundation creates an owned UTF-8 `NSData`; Rust copies its bytes and moves them into an owned `String`.

Write copies borrowed Rust UTF-8 text into an `NSString`, then assigns `UIPasteboard.string`. Foundation/UIKit copy the assigned string. Apple documents that setting `string` replaces all current pasteboard items with the new item, so other item representations are not preserved.

Clear assigns an empty array to `UIPasteboard.items`. Apple documents that setting `items` replaces all current pasteboard items; this removes plain text and all other current representations. Both write and clear affect shared pasteboard state and can race with other apps.

The backend reports `Availability::Available`; UIKit does not provide a permission-state query for general-pasteboard access. It does not claim a stable permission state or map system privacy UI into a permission enum. Native conversion errors map to `InvalidInput`; an observed `hasStrings`/`string` race maps to `Platform`. UIKit methods used here do not return an error result.

## Privacy, API floor, and configuration

Starting in iOS 14, the system notifies the user when an app reads general-pasteboard content from another app without user intent. Starting in iOS 16, programmatic pasting can raise a system alert asking the user to approve access. This backend performs programmatic reads and does not avoid either system behavior. UIKit's `UIPasteControl` offers a user-initiated paste path without a prompt, but requires app UI integration and is out of scope.

The installed Xcode 26.6 / iPhoneOS 26.5 SDK marks `UIPasteboard` as available from iOS 3.0 and `hasStrings` from iOS 10.0; the required `hasStrings` preflight sets the backend floor at iOS 10.0. The public SDK headers and Apple documentation reviewed do not specify an `Info.plist` usage-description key or entitlement for this general-pasteboard path. The backend adds neither; system privacy behavior is not an app permission grant.

Apple references: [`UIPasteboard`](https://developer.apple.com/documentation/uikit/uipasteboard), [`UIPasteboard.strings`](https://developer.apple.com/documentation/uikit/uipasteboard/strings), [`UIPasteboard.string`](https://developer.apple.com/documentation/uikit/uipasteboard/string), [`UIPasteboard.items`](https://developer.apple.com/documentation/uikit/uipasteboard/items), [`UIPasteControl`](https://developer.apple.com/documentation/uikit/uipastecontrol).

## Validation evidence and limits

Host tests exercise UTF-8 conversion and the deferred-operation seam with a fake backend. They do not read or mutate the developer's live general pasteboard. Device/simulator target checks and minimal-link import inspection establish build and linkage evidence only; no simulator privacy prompt, paste action, or live pasteboard mutation was performed.
