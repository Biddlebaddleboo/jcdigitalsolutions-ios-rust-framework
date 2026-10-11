# iOS UIKit presentation

`ios-presentation` is a partial UIKit system-UI backend. It adds a synchronous one-action
acknowledgement alert and a two-action destructive confirmation sheet; it is not a general
presentation, navigation, arbitrary action-list, custom-controller, or SwiftUI API.

This alert API is separate from [the iOS share API](sharing.md): it accepts no `ShareItem`, creates
no `UIActivityViewController`, exposes no `ShareOutcome` or share completion callback, and makes no
recipient-delivery claim. B7's share callback and F7's opt-in C ABI callback can report a UIKit
share result when one arrives; B9 instead reports only bounded preflight errors or that it issued
the void UIKit request. B9 does not report the alert action or its dismissal to Rust or the host.

## Present an acknowledgement

The caller supplies a borrowed, caller-owned `UIViewController`, a typed main-thread proof, and
borrowed UTF-8 title, message, and action text. The crate creates a UIKit `UIAlertController` with
alert style and exactly one `UIAlertActionStyleDefault` action. The action has no handler, so no
Rust callback or result is produced; the action only dismisses the alert. The crate does not observe
whether the user selects it.

```rust
use ios_presentation::{present_acknowledgement, PresentationError};
use ios_runtime::main_thread::MainThread;
use objc2_ui_kit::UIViewController;

fn show_acknowledgement(
    main_thread: MainThread,
    presenter: &UIViewController,
) -> Result<(), PresentationError> {
    present_acknowledgement(
        main_thread,
        presenter,
        "Saved",
        "Your changes are ready.",
        "OK",
    )
}
```

The caller obtains the proof with `MainThread::current()`. The crate does not create an application,
window, presenter, or view tree, keep the borrowed presenter or context after the call, or return an
alert/controller handle. If UIKit accepts and presents the request, UIKit manages the presented
alert; the caller retains responsibility for supplying valid presentation context and managing its
application/controller lifecycle. UIKit values remain main-thread-only and are not exposed as
`Send` or `Sync` state.

## Preflight and result limits

Before constructing the alert, preflight checks only the presenter's observable state: whether the
presenter is being presented or dismissed, whether it already presents another controller, whether
its view is already loaded, and whether that loaded view belongs to a window. Preflight never loads
the view. A known-invalid state returns one of `PresentationError::PresenterTransitioning`,
`PresentationError::AlreadyPresenting`, `PresentationError::ViewNotLoaded`, or
`PresentationError::ViewNotInWindow`.

`UIViewController.presentViewController:animated:completion:` returns `void` and has no error
callback. UIKit also offers an optional completion block, called after the presented controller's
`viewDidAppear:`; B9 passes `None` for that block and supplies no action handler. Therefore
`Ok(())` means only that the presentation request was issued. It does not prove that UIKit displayed
the alert, that the user selected the action, or that dismissal completed. B9 exposes no signal for
presentation-transition completion, action selection, or dismissal. UIKit can refuse a request for
conditions not caught by this bounded preflight, and the presenter hierarchy can change after
preflight; neither case has a recovery signal in this API. No live alert, display, acknowledgement,
dismissal, or device/simulator behavior is claimed.

## Present a destructive confirmation sheet

`present_destructive_confirmation` creates a public UIKit `UIAlertController` with
`UIAlertControllerStyle::ActionSheet` and exactly two actions: one caller-titled destructive action
and one caller-titled cancel action. It accepts no arbitrary action list, result future, controller
handle, navigation operation, or app-specific behavior.

```rust
use ios_presentation::{
    DestructiveConfirmationStartError, DestructiveConfirmationText, PopoverAnchor,
    present_destructive_confirmation,
};
use ios_runtime::main_thread::MainThread;
use objc2_core_foundation::CGRect;
use objc2_ui_kit::{UIView, UIViewController};

fn request_confirmation<F>(
    main_thread: MainThread,
    presenter: &UIViewController,
    source_view: &UIView,
    on_destructive: F,
) -> Result<(), DestructiveConfirmationStartError<F>>
where
    F: FnOnce() + 'static,
{
    present_destructive_confirmation(
        main_thread,
        presenter,
        PopoverAnchor::new(source_view, CGRect::ZERO),
        DestructiveConfirmationText::new(
            "Delete item?",
            "This cannot be undone.",
            "Delete",
            "Cancel",
        ),
        on_destructive,
    )
}
```

The caller supplies a typed `MainThread` proof; a borrowed, caller-owned `UIViewController`; a
borrowed source `UIView` and explicit `CGRect` in that view's coordinate space through
`PopoverAnchor`; and UTF-8 title, message, destructive-action, and cancel-action text. Rust borrows
the presenter and anchor only for the synchronous call and stores neither in callback state. The
UIKit `sourceView` property is strong, so UIKit applies its own native retain/release behavior to
the anchor when it configures an iPad popover. The caller remains responsible for the live presenter,
window hierarchy, and application lifecycle.

The bounded preflight does not load a view. It preserves the acknowledgement checks for a presenter
transition, an already-presented controller, a not-yet-loaded presenter view, and a presenter view
without a window. It also rejects a source rectangle with non-finite coordinates or negative width
or height, an anchor view without a window, and an anchor in a different window from the presenter.
An all-zero source rectangle is accepted. On iPad, the crate requires a UIKit popover presentation
controller and sets both `sourceView` and `sourceRect` before presentation; without that configuration
the request could raise UIKit's missing-anchor exception. On compact phone UI, UIKit adapts the
action sheet modally; the caller's anchor is still preflighted but not applied as popover state.

Known-invalid preflight or missing iPad popover configuration returns
`DestructiveConfirmationStartError<F>`. `into_parts()` returns the
`DestructiveConfirmationError` and the exact uncalled `FnOnce` callback, so a rejected request does
not consume the caller's operation. Presenter errors wrap the same four `PresentationError`
variants used by the acknowledgement API. After
`Ok(())`, the callback has moved into the destructive action's escaping Objective-C block. UIKit
invokes it on the main thread only when the destructive action is selected. The cancel action and
interactive dismissal do not invoke it. The callback state is one-shot even if its block is called
again, and it drops with the block if UIKit releases the action/controller without selection; the
crate uses no global registry. Callback captures may remain alive as long as UIKit retains the
action. Avoid a strong callback capture of the presenting controller when that would create a
presenter → alert → action → callback retain cycle.

A callback panic is caught before the block returns to UIKit when the process uses an unwinding Rust
panic strategy; `panic=abort` still aborts. `Ok(())` means only that the void UIKit presentation
request was issued. It does not prove that the sheet appeared, either action ran, or dismissal
completed. UIKit can refuse a request for a condition outside preflight, and the view/window state
can change after preflight. No live action-sheet display, selection, cancellation, dismissal, device,
Simulator, or VoiceOver behavior has been verified.

## API floor and configuration

The active public iPhoneOS SDK is Xcode 26.6 (build 17F113), iOS SDK 26.5. Its `UIAlertController.h`
marks `UIAlertController`, `UIAlertAction`, both alert styles, and the action styles used here as
available from iOS 8.0. The popover source-view/rectangle APIs are also available from iOS 8.0;
`UIViewController` presentation, `presentedViewController`, `isBeingPresented`, and
`isBeingDismissed` are available from iOS 5.0. The shared preflight uses
`UIViewController.viewIfLoaded`, which the SDK header marks available from iOS 9.0, so the
effective public API floor for both paths remains iOS 9.0.

The installed SDK's `SDKSettings.plist` sets `MinimumDeploymentTarget` to iOS 12.0 and
`DefaultDeploymentTarget` to iOS 26.5. These SDK settings are not the API floor. The crate does not
set a deployment target in its manifest.

The selected public bindings are `objc2-ui-kit` for `UIAlertController`, `UIAlertAction`,
`UIDevice`, `UIPopoverPresentationController`, `UIViewController`, `UIView`, and `UIWindow`,
`objc2-core-foundation` for `CGRect`, `objc2-foundation` for owned `NSString` values, and `block2`
for the retained destructive-action handler. No Swift source, private selector, custom controller,
permission prompt, or entitlement is added. A temporary external Rust consumer linked for device at
iOS 12.0 and Simulator at iOS 14.0 with SDK 26.5. `otool -L` reports UIKit, Foundation,
`libSystem.B.dylib`, and `libobjc.A.dylib` as its direct framework/runtime imports; `nm -u` shows
`_objc_msgSend` and `_objc_retainAutoreleasedReturnValue` and no Swift or Python runtime symbols.
`vtool -show-build`
confirms those probe deployment settings. The probe was not executed and does not establish
application packaging or runtime presentation behavior.

The B518 action-sheet probe is built by
`sh platform/ios/ios-presentation/check-link-imports.sh` for device at deployment target iOS 12.0
and Simulator at iOS 14.0 with SDK 26.5. Both Release artifacts import CoreFoundation, Foundation,
UIKit, `libSystem.B.dylib`, and `libobjc.A.dylib`; `nm -u` includes `__Block_copy`,
`__Block_release`, `__NSConcreteStackBlock`, `_objc_getClass`, and `_objc_msgSend`, with no Swift or
Python runtime symbols. `vtool -show-build` confirms minos 12.0/SDK 26.5 for device and
minos 14.0/SDK 26.5 for Simulator. The script checks the public action, popover, preflight, and
presentation selectors, and neither artifact was executed.

Apple references: [`UIAlertController`](https://developer.apple.com/documentation/uikit/uialertcontroller),
[`UIAlertAction`](https://developer.apple.com/documentation/uikit/uialertaction),
[`UIViewController.present`](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29),
[`UIViewController.viewIfLoaded`](https://developer.apple.com/documentation/uikit/uiviewcontroller/viewifloaded),
and [`UIViewController.isBeingPresented`](https://developer.apple.com/documentation/uikit/uiviewcontroller/isbeingpresented).

Device and simulator target checks establish compile/lint evidence; the B9 consumer and B518 action
sheet probe add compile/link and direct-import evidence only. None of these checks runs an app,
presents or dismisses UIKit UI, establishes that the caller's hierarchy is ready, or proves UIKit
presentation behavior.
