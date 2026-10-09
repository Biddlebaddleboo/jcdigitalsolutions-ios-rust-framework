use ios_runtime::main_thread::MainThread;
use objc2::rc::autoreleasepool;
use objc2_foundation::NSString;
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle, UIViewController,
};

/// A bounded reason the supplied presenter is known not to be ready for an alert request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationError {
    /// The presenter is currently being presented or dismissed.
    PresenterTransitioning,
    /// The presenter already presents another controller.
    AlreadyPresenting,
    /// The presenter's view has not been loaded; preflight did not load it.
    ViewNotLoaded,
    /// The presenter's loaded view is not attached to a window.
    ViewNotInWindow,
}

/// Request a system-owned, one-action acknowledgement alert from a caller-owned presenter.
///
/// The caller retains ownership of `presenter` and controls its view tree and lifecycle. This
/// function synchronously constructs an alert with exactly one default action, passes no action
/// handler or presentation-completion block, and sends a presentation request. UIKit manages the
/// alert only if it accepts and presents the request. The proof and UIKit objects are
/// main-thread-only; this function exposes no retained UIKit handle or signal that UIKit accepted
/// the request.
///
/// `Ok(())` means only that `presentViewController:animated:completion:` was called. That UIKit
/// method returns `void` and has no error callback, so this result does not prove that the alert
/// appeared, remained visible, was acknowledged, or completed. Preflight cannot recover from a
/// UIKit refusal or a hierarchy race after the checks.
pub fn present_acknowledgement(
    main_thread: MainThread,
    presenter: &UIViewController,
    title: &str,
    message: &str,
    action_title: &str,
) -> Result<(), PresentationError> {
    autoreleasepool(|_| {
        preflight(presenter)?;
        let marker = main_thread.into_objc2();
        let title = NSString::from_str(title);
        let message = NSString::from_str(message);
        let action_title = NSString::from_str(action_title);
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&title),
            Some(&message),
            UIAlertControllerStyle::Alert,
            marker,
        );
        let action = UIAlertAction::actionWithTitle_style_handler(
            Some(&action_title),
            UIAlertActionStyle::Default,
            None,
            marker,
        );
        alert.addAction(&action);
        presenter.presentViewController_animated_completion(&alert, true, None);
        Ok(())
    })
}

fn preflight(presenter: &UIViewController) -> Result<(), PresentationError> {
    if presenter.isBeingPresented() || presenter.isBeingDismissed() {
        return Err(PresentationError::PresenterTransitioning);
    }
    if presenter.presentedViewController().is_some() {
        return Err(PresentationError::AlreadyPresenting);
    }
    let view = presenter
        .viewIfLoaded()
        .ok_or(PresentationError::ViewNotLoaded)?;
    if view.window().is_none() {
        return Err(PresentationError::ViewNotInWindow);
    }
    Ok(())
}
