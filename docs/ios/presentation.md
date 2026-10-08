# iOS acknowledgement presentation

`ios-presentation` is a partial UIKit system-UI backend. It adds one synchronous path for a
one-action acknowledgement alert; it is not a general presentation, navigation, action-sheet,
popover, custom-controller, or SwiftUI API.

## Present an acknowledgement

The caller supplies a borrowed, caller-owned `UIViewController`, a typed main-thread proof, and
borrowed UTF-8 title, message, and action text. The crate creates a UIKit `UIAlertController` with
alert style and exactly one `UIAlertActionStyleDefault` action. The action has no handler, so no
Rust callback or result is produced; UIKit handles the acknowledgement and dismisses the alert.

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
alert/controller handle. UIKit owns the presented alert after the request; the caller retains
responsibility for supplying valid presentation context and managing its application/controller
lifecycle. UIKit values remain main-thread-only and are not exposed as `Send` or `Sync` state.

## Preflight and result limits

Before constructing the alert, preflight checks only the presenter's observable state: whether the
presenter is being presented or dismissed, whether it already presents another controller, whether
its view is already loaded, and whether that loaded view belongs to a window. Preflight never loads
the view. A known-invalid state returns one of `PresentationError::PresenterTransitioning`,
`PresentationError::AlreadyPresenting`, `PresentationError::ViewNotLoaded`, or
`PresentationError::ViewNotInWindow`.

`UIViewController.presentViewController:animated:completion:` returns `void` and has no error
callback. Therefore `Ok(())` means only that the presentation request was issued. It does not prove
that UIKit displayed the alert, that the user selected the action, or that dismissal completed.
UIKit can refuse a request for conditions not caught by this bounded preflight, and the presenter
hierarchy can change after preflight; neither case has a recovery signal in this API. No live alert,
display, acknowledgement, dismissal, or device/simulator behavior is claimed.

## API floor and configuration

The active public iPhoneOS SDK is Xcode 26.6 (build 17F113), iOS SDK 26.5. Its `UIAlertController.h`
marks `UIAlertController` as available from iOS 8.0; `UIViewController.presentViewController` is
available from iOS 5.0. The preflight uses `UIViewController.viewIfLoaded`, which the SDK header
marks available from iOS 9.0, so the effective public API floor for this crate is iOS 9.0. The
installed SDK's `SDKSettings.plist` lists iOS 12.0 as its lowest suggested deployment target and
26.5 as its default deployment target. These SDK settings are not the API floor. The crate does not
set a deployment target in its manifest.

The selected public bindings are `objc2-ui-kit` for `UIAlertController`, `UIAlertAction`,
`UIViewController`, `UIView`, and `UIWindow`, and `objc2-foundation` for owned `NSString` values.
No Swift source, private selector, custom action handler, permission prompt, or entitlement is
added. The checked SDK headers are in the installed public UIKit framework; imports and live
runtime behavior were not independently inspected with a temporary consumer executable.

Apple references: [`UIAlertController`](https://developer.apple.com/documentation/uikit/uialertcontroller),
[`UIAlertAction`](https://developer.apple.com/documentation/uikit/uialertaction),
[`UIViewController.present`](https://developer.apple.com/documentation/uikit/uiviewcontroller/present%28_%3Aanimated%3Acompletion%3A%29),
[`UIViewController.viewIfLoaded`](https://developer.apple.com/documentation/uikit/uiviewcontroller/viewifloaded),
and [`UIViewController.isBeingPresented`](https://developer.apple.com/documentation/uikit/uiviewcontroller/isbeingpresented).

Device and simulator checks establish compile/lint evidence only. They do not run an app, present or
dismiss an alert, establish that the caller's hierarchy is ready, or prove UIKit presentation
behavior.
