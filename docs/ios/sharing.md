# iOS plain-text clipboard

`ios-sharing` implements the `framework-sharing::ClipboardBackend` contract with UIKit's systemwide general `UIPasteboard`. It supports plain-text read, write, and clear only. It does not include a share sheet, `UIPasteControl`, named pasteboards, or rich representations.

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
