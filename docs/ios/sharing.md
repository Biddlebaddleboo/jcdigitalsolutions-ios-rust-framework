# iOS sharing

This guide covers the `ios-sharing` crate's plain-text clipboard backend and outgoing system-share
backend. The clipboard section below retains its existing behavior and privacy notes; the share
backend is a separate UIKit presentation path.

The crate has `clipboard` and `share` Cargo features, both enabled by default to preserve existing
Rust consumers. A consumer may disable default features and select one backend. The F6 C clipboard
feature selects only `ios-sharing/clipboard`, so it does not compile the B7 share module or enable
share-sheet UIKit and Blocks APIs. This crate feature split changes no D5, D6, B6, or B7 operation
semantics.

## Outgoing system share

`IosShareBackend` implements `framework_sharing::ShareBackend` with UIKit's public
`UIActivityViewController`. It supports only owned UTF-8 text and URL-text items. It does not
support files, images, rich representations, custom activities, recipient data, activity IDs,
previews, extensions, clipboard operations, or a delivery guarantee.

`IosShareSession` provides the same presentation and result mapping through a callback API for
foreign-function callers. It does not change the `ShareBackend` future contract.

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

For callback-based use, create `IosShareSession` with retained UIKit objects and the same explicit
anchor. The session owns those references and permits one accepted share at a time. `start` presents
on the call and returns URL and preflight errors synchronously. A rejected start returns the
unconsumed callback in `ShareStartError::callback` with its error in `ShareStartError::error`; the
caller keeps or drops that callback. Once accepted, the session owns it until terminal result,
cancel, or drop. If UIKit invokes its completion handler, the one-shot callback runs on the main
thread. UIKit's presentation method has no error
result, so `Ok(())` means the presentation call was made, not that visible UI or a terminal callback
is guaranteed. If an operation is still active, a second `start` returns
`ErrorKind::AlreadyExists`; after a terminal callback, a later `start` first clears the completed
operation state. `availability()` remains `Availability::Unknown` and does not preflight or present
UI.

~~~rust
use framework_sharing::{ShareError, ShareOutcome, ShareRequest};
use ios_runtime::main_thread::MainThread;
use ios_sharing::{IosShareSession, ShareStartError};
use objc2::rc::Retained;
use objc2_core_foundation::CGRect;
use objc2_ui_kit::{UIView, UIViewController};

fn start_share<F>(
    main_thread: MainThread,
    presenter: Retained<UIViewController>,
    source_view: Retained<UIView>,
    source_rect: CGRect,
    request: ShareRequest,
    completion: F,
) -> Result<IosShareSession, ShareStartError<F>>
where
    F: FnOnce(Result<ShareOutcome, ShareError>) + 'static,
{
    let mut session = IosShareSession::new(main_thread, presenter, source_view, source_rect);
    session.start(request, completion)?;
    Ok(session)
}
~~~

Keep the session alive while an accepted operation is active and call `cancel()` or drop the
session on the main thread to detach its result callback. `cancel()` returns `true` only when it
detaches an active operation; after completion or a prior cancel it returns `false`. Cancellation
and drop suppress late callbacks and clear the UIKit completion handler, but do not promise to
dismiss share UI already on screen. The session and its callback state are `!Send` and `!Sync`.

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

- UIKit's native callback is accepted at most once
- The future path clears the UIKit completion block before its stored waker wakes on main
- The session path clears the block and detaches callback state before the app callback
- Native result data goes to main by an async queue post; the app callback runs once on main after
  `IosShareSession::start` returns
- Callback state access and UIKit cleanup stay on main; a callback panic is caught at the main-queue
  boundary
- With `panic=abort`, a panic aborts the process with no unwind
- A future drop before its first poll starts no native work; after presentation, it clears the
  waker/result state and UIKit completion block, so late native callbacks have no effect
- The system UI may stay visible; a drop or cancel does not promise dismissal or rollback

The backend adds no permission prompt, app-specific usage string, entitlement, executor, global
registry, or Swift source. No `Info.plist` key or entitlement was identified for constructing and
presenting `UIActivityViewController`; the selected destination activity may have separate behavior
or host-app requirements, which this backend does not configure.

### API floor and evidence limits

The UIKit API floor is iOS 8.0: `UIActivityViewController` itself is available from iOS 6.0, while
`completionWithItemsHandler`, `UIPopoverPresentationController`, and popover presentation are
available from iOS 8.0. `UIViewController.present` is available from iOS 5.0. The current Rust iOS
device target emits a deployment minimum of iOS 10.0, and its simulator target emits iOS 14.0; these
are the practical minimums of the linked Rust artifacts, above the UIKit API floor. The active host
has Xcode 26.6 (build 17F113), iPhoneOS SDK 26.5, and iPhoneSimulator SDK 26.5; this remains below
the repository's planned Xcode 27.x baseline. The local SDK headers and Apple's public references
were checked for these APIs and floors. No device-only availability restriction was found in the
reviewed public UIKit declarations. Device and simulator consumers both link, but no simulator or
device UI action was run.

Apple references: [`UIActivityViewController`](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller),
[`UIActivityViewController` initializer](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/init%28activityitems%3Aapplicationactivities%3A%29),
[`completionWithItemsHandler`](https://developer.apple.com/documentation/uikit/uiactivityviewcontroller/completionwithitemshandler-swift.property),
[`UIViewController.present`](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29),
[`UIPopoverPresentationController.sourceView`](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourceview),
[`UIPopoverPresentationController.sourceRect`](https://developer.apple.com/documentation/uikit/uipopoverpresentationcontroller/sourcerect),
and [`NSURL.URLWithString:`](https://developer.apple.com/documentation/foundation/nsurl/urlwithstring%3A).

Host fake-backend checks, target checks, and link/import inspection are not live share-sheet evidence.
They do not establish simulator/device UI behavior, recipient delivery, parity, or performance.
The B7-only Release link/import/minos inspection is reproducible with
`sh platform/ios/ios-sharing/check-share-link-imports.sh`; it builds and inspects a share-only
consumer for iOS device and simulator targets but does not execute it. The gate imports exactly
CoreFoundation, Foundation, UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`, retains the expected
share selectors, excludes the clipboard feature and Swift/Python runtime symbols, and records
device minos 10.0 and Simulator minos 14.0 with SDK 26.5. The gate is wired in macOS CI; no passing
workflow run is recorded. No UI or recipient behavior is established.

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

For B84, call `clipboard.backend().has_plain_text()` for a synchronous presence snapshot when a caller only needs to know whether string data is present. It maps to UIKit's `UIPasteboard.hasStrings`, returns a bool rather than a count, does not load or copy pasteboard contents, and is not a permission or read-success check. Call it on the backend's main thread. The shared pasteboard can change after the snapshot. Apple lists `hasStrings` among type-checking APIs that avoid user notifications and alerts when the system has not established user intent; a later call to `Clipboard::read()` still follows the programmatic-read privacy behavior below.

## Reads, writes, clear, and copies

Read first checks `UIPasteboard.hasStrings` and only then loads `UIPasteboard.strings`. The backend copies the first string value in that array, including when an earlier pasteboard item has no string representation. No string type returns `Ok(None)`. If another app changes the shared pasteboard between the preflight and value read, or Foundation cannot encode the native string as UTF-8, the backend returns `ClipboardError::Backend` rather than reporting no value. Foundation creates an owned UTF-8 `NSData`; Rust copies its bytes and moves them into an owned `String`. Unlike `has_plain_text()`, this content read may trigger system privacy UI.

Write copies borrowed Rust UTF-8 text into an `NSString`, then assigns `UIPasteboard.string`. Foundation/UIKit copy the assigned string. Apple documents that setting `string` replaces all current pasteboard items with the new item, so other item representations are not preserved.

Clear assigns an empty array to `UIPasteboard.items`. Apple documents that setting `items` replaces all current pasteboard items; this removes plain text and all other current representations. Both write and clear affect shared pasteboard state and can race with other apps.

The backend reports `Availability::Unknown`: UIKit provides no query for current programmatic-read usability or stable permission state for general-pasteboard access. `Unknown` means the backend cannot determine current usability; it does not mean `Availability::RequiresPermission`. Programmatic reads may trigger system privacy UI, which this backend does not map to a permission enum. Native conversion errors map to `InvalidInput`; an observed `hasStrings`/`strings` race maps to `Platform`. UIKit methods used here do not return an error result.

## Privacy, API floor, and configuration

Starting in iOS 14, the system notifies the user when an app reads general-pasteboard content from another app without user intent. Starting in iOS 16, programmatic pasting can raise a system alert asking the user to approve access. This backend performs programmatic reads and does not avoid either system behavior. UIKit's `UIPasteControl` offers a user-initiated paste path without a prompt, but requires app UI integration and is out of scope.

The installed Xcode 26.6 / iPhoneOS 26.5 SDK marks `UIPasteboard` as available from iOS 3.0 and `hasStrings` from iOS 10.0; the required `hasStrings` preflight sets the backend floor at iOS 10.0. The public SDK headers and Apple documentation reviewed do not specify an `Info.plist` usage-description key or entitlement for this general-pasteboard path. The backend adds neither; system privacy behavior is not an app permission grant.

Apple references: [`UIPasteboard`](https://developer.apple.com/documentation/uikit/uipasteboard), [`UIPasteboard.hasStrings`](https://developer.apple.com/documentation/uikit/uipasteboard/hasstrings), [`UIPasteboard.strings`](https://developer.apple.com/documentation/uikit/uipasteboard/strings), [`UIPasteboard.string`](https://developer.apple.com/documentation/uikit/uipasteboard/string), [`UIPasteboard.items`](https://developer.apple.com/documentation/uikit/uipasteboard/items), [`UIPasteControl`](https://developer.apple.com/documentation/uikit/uipastecontrol).

## Validation evidence and limits

B84 non-test validation passed for both device and Simulator with the clipboard-only feature:

~~~sh
cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios
cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios-sim
cargo clippy --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios-sim -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios
RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios-sim
sh platform/ios/ios-sharing/check-clipboard-link-imports.sh
~~~

The link/import gate builds but does not execute its link-only example. It passed with exactly
UIKit, Foundation, `libobjc.A.dylib`, and `libSystem.B.dylib` imported for each target; it verified
the `hasStrings` marker, no share or language-runtime symbols, device minos 10.0, and Simulator
minos 14.0 with SDK 26.5. Rustdoc output includes `IosClipboardBackend::has_plain_text()` for both
targets. No tests, live pasteboard access, or consumer/probe execution occurred during this B84
validation.

Host tests exercise UTF-8 conversion, error propagation, and the deferred-operation seam with a fake backend. `cargo test --locked --offline -p ios-sharing --no-default-features --features clipboard` passed four B6 host tests; these do not read or mutate the developer's live general pasteboard.

The clipboard-only target configuration passed device and simulator checks. A temporary minimal consumer under ignored `target/b6-link-probe` was built for both targets with:

~~~sh
IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b6-link-probe/Cargo.toml --release --target aarch64-apple-ios
IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --offline --manifest-path target/b6-link-probe/Cargo.toml --release --target aarch64-apple-ios-sim
~~~

`otool -L` showed only UIKit, Foundation, `libobjc.A`, and `libSystem.B`; no Swift runtime or unrelated framework was imported. The device binary load metadata records minos 10.0; the simulator records minos 14.0. The probe binary contains the `UIPasteboard`, `hasStrings`, `setString:`, and `setItems:` names and no `UIActivityViewController` or `UIActivity` symbols, confirming the clipboard-only feature did not link the share-sheet code. The probe was built but never run; no simulator privacy prompt, paste action, or live pasteboard mutation was performed. These checks used Xcode 26.6 and iOS/iOS Simulator SDK 26.5, below the planned Xcode 27.x baseline.

The persistent B6 clipboard-only Release gate additionally checks the `generalPasteboard`, `strings`,
`firstObject`, and `dataUsingEncoding:` selectors, so the linked consumer is asserted to use the
systemwide general pasteboard and documented first-item UTF-8 read path as well as its type preflight,
write, and clear selectors. The updated assertions passed in the integrated Release gate for device
and Simulator; the probes were not executed.

The integrated B6 report in `PLAN_IOS_CLIPBOARD.md` also records these passing gates:

~~~sh
cargo fmt --all -- --check
cargo test --locked --offline -p ios-sharing
cargo check --locked --offline -p ios-sharing --target aarch64-apple-ios
cargo check --locked --offline -p ios-sharing --target aarch64-apple-ios-sim
cargo clippy --locked --offline --all-targets -p ios-sharing --target aarch64-apple-ios -- -D warnings
cargo clippy --locked --offline --all-targets -p ios-sharing --target aarch64-apple-ios-sim -- -D warnings
cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios
cargo check --locked --offline -p ios-sharing --no-default-features --features clipboard --target aarch64-apple-ios-sim
git diff --check
~~~

The full crate test run passed 14 tests, including the separate share module and two B7 callback completion-state tests. The report also says clipboard-only strict Clippy passed on both targets, but does not preserve those exact feature-only Clippy command lines. This is integrated feature-split package evidence; the older pre-feature-split branch at `c2eea90` had a three-test clipboard seam and no branch-local command or import output, so its gates remain unverified by this record.
