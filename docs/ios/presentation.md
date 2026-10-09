# iOS acknowledgement presentation

`ios-presentation` is a partial UIKit system-UI backend. It adds one synchronous path for a
one-action acknowledgement alert; it is not a general presentation, navigation, action-sheet,
popover, custom-controller, or SwiftUI API.

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

## API floor and configuration

The active public iPhoneOS SDK is Xcode 26.6 (build 17F113), iOS SDK 26.5. Its `UIAlertController.h`
marks `UIAlertController` and `UIAlertAction` as available from iOS 8.0; `UIViewController`
presentation, `presentedViewController`, `isBeingPresented`, and `isBeingDismissed` APIs used here
are available from iOS 5.0. The preflight uses `UIViewController.viewIfLoaded`, which the SDK
header marks available from iOS 9.0, so the effective public API floor for this crate is iOS 9.0.
The installed SDK's `SDKSettings.plist` sets `MinimumDeploymentTarget` to iOS 12.0 and
`DefaultDeploymentTarget` to iOS 26.5. These SDK settings are not the API floor. The crate does not
set a deployment target in its manifest.

The selected public bindings are `objc2-ui-kit` for `UIAlertController`, `UIAlertAction`,
`UIViewController`, `UIView`, and `UIWindow`, and `objc2-foundation` for owned `NSString` values.
No Swift source, private selector, custom action handler, permission prompt, or entitlement is
added. A temporary external Rust consumer linked for device at iOS 12.0 and Simulator at iOS 14.0
with SDK 26.5. `otool -L` reports UIKit, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib` as
its direct framework/runtime imports; `nm -u` shows `objc_msgSend` and
`objc_retainAutoreleasedReturnValue` and no Swift or Python runtime symbols. `vtool -show-build`
confirms those probe deployment settings. The probe was not executed and does not establish
application packaging or runtime presentation behavior.

Apple references: [`UIAlertController`](https://developer.apple.com/documentation/uikit/uialertcontroller),
[`UIAlertAction`](https://developer.apple.com/documentation/uikit/uialertaction),
[`UIViewController.present`](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29),
[`UIViewController.viewIfLoaded`](https://developer.apple.com/documentation/uikit/uiviewcontroller/viewifloaded),
and [`UIViewController.isBeingPresented`](https://developer.apple.com/documentation/uikit/uiviewcontroller/isbeingpresented).

Device and simulator target checks establish compile/lint evidence; the consumer probe adds
compile/link and direct-import evidence only. None of these checks runs an app, presents or dismisses
an alert, establishes that the caller's hierarchy is ready, or proves UIKit presentation behavior.
