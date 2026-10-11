use ios_runtime::main_thread::MainThread;
use objc2::rc::autoreleasepool;
use objc2_core_foundation::CGRect;
use objc2_foundation::NSString;
use objc2_ui_kit::UIDevice;
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle,
    UIUserInterfaceIdiom, UIView, UIViewController, UIWindow,
};
use std::ptr;

use crate::callback::{OneShot, catch_foreign_panic, return_callback_on_error};
use crate::policy::valid_popover_source_rect;

/// A bounded reason the supplied UIKit presentation context is known to be invalid.
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

/// A bounded reason a destructive confirmation request is known to be invalid before presentation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DestructiveConfirmationError {
    /// The existing acknowledgement presenter preflight rejected the context.
    Presenter(PresentationError),
    /// The popover source rectangle has non-finite coordinates or negative dimensions.
    InvalidPopoverAnchor,
    /// The supplied popover source view is not attached to a window.
    AnchorViewNotInWindow,
    /// The supplied popover source view is attached to a different window than the presenter.
    AnchorViewInDifferentWindow,
    /// UIKit did not provide the popover configuration required for an iPad action sheet.
    PopoverUnavailable,
}

/// Caller-owned source view and its rectangle in that view's coordinate system.
///
/// `source_view` is borrowed only for the call. UIKit's popover controller applies its native
/// ownership rules to the configured view; the caller remains responsible for its hierarchy and
/// lifecycle.
pub struct PopoverAnchor<'a> {
    source_view: &'a UIView,
    source_rect: CGRect,
}

impl<'a> PopoverAnchor<'a> {
    /// Create an explicit source-view and source-rectangle configuration.
    pub fn new(source_view: &'a UIView, source_rect: CGRect) -> Self {
        Self {
            source_view,
            source_rect,
        }
    }
}

/// Caller-provided UTF-8 text for the two-action destructive confirmation sheet.
pub struct DestructiveConfirmationText<'a> {
    title: &'a str,
    message: &'a str,
    destructive_action_title: &'a str,
    cancel_action_title: &'a str,
}

impl<'a> DestructiveConfirmationText<'a> {
    /// Create title, message, destructive-action, and cancel-action text.
    pub fn new(
        title: &'a str,
        message: &'a str,
        destructive_action_title: &'a str,
        cancel_action_title: &'a str,
    ) -> Self {
        Self {
            title,
            message,
            destructive_action_title,
            cancel_action_title,
        }
    }
}

/// A known-invalid presentation request that still owns the uncalled destructive callback.
#[must_use = "recover or drop the uncalled destructive callback"]
pub struct DestructiveConfirmationStartError<F> {
    error: DestructiveConfirmationError,
    on_destructive: F,
}

impl<F> DestructiveConfirmationStartError<F> {
    /// Return the bounded preflight or popover-configuration error.
    pub fn error(&self) -> DestructiveConfirmationError {
        self.error
    }

    /// Recover the error and the callback, which has not been invoked or transferred to UIKit.
    pub fn into_parts(self) -> (DestructiveConfirmationError, F) {
        (self.error, self.on_destructive)
    }
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
    presenter_window(presenter).map(drop)
}

/// Request a destructive confirmation action sheet from a caller-owned presenter.
///
/// The caller supplies a typed main-thread proof, a borrowed presenter, a borrowed popover anchor,
/// and `DestructiveConfirmationText` for the title, message, destructive action, and cancel action.
/// The sheet has exactly those two actions. Its destructive handler owns `on_destructive` after the
/// request is issued; UIKit calls it on the main thread only when that action is selected. The
/// cancel action and interactive dismissal do not invoke it. The closure must be `'static` because
/// UIKit may retain its action after this function returns; it need not be `Send` or `Sync`.
///
/// Known-invalid presenter or anchor state, or a missing iPad popover controller, returns the
/// uncalled callback inside `DestructiveConfirmationStartError`. Once `Ok(())` is returned, the
/// callback belongs to the UIKit action block and is invoked at most once. If UIKit releases the
/// action without selection, the uncalled closure state drops with that block; this API uses no
/// global callback registry. Panics from the closure are caught at the Objective-C block boundary
/// when the Rust panic strategy unwinds; `panic=abort` still aborts.
///
/// `Ok(())` means only that `presentViewController:animated:completion:` was called. UIKit's
/// presentation method returns `void` and has no error callback, so this result does not prove that
/// the sheet appeared, that the user selected either action, or that dismissal completed. A UIKit
/// refusal or a hierarchy race after preflight has no recovery signal in this API.
pub fn present_destructive_confirmation<F>(
    main_thread: MainThread,
    presenter: &UIViewController,
    anchor: PopoverAnchor<'_>,
    text: DestructiveConfirmationText<'_>,
    on_destructive: F,
) -> Result<(), DestructiveConfirmationStartError<F>>
where
    F: FnOnce() + 'static,
{
    let mut on_destructive = Some(on_destructive);
    let result = autoreleasepool(|_| {
        preflight_destructive(presenter, &anchor)?;

        let marker = main_thread.into_objc2();
        let is_ipad =
            UIDevice::currentDevice(marker).userInterfaceIdiom() == UIUserInterfaceIdiom::Pad;
        let title = NSString::from_str(text.title);
        let message = NSString::from_str(text.message);
        let destructive_action_title = NSString::from_str(text.destructive_action_title);
        let cancel_action_title = NSString::from_str(text.cancel_action_title);
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&title),
            Some(&message),
            UIAlertControllerStyle::ActionSheet,
            marker,
        );

        if is_ipad {
            let Some(popover) = alert.popoverPresentationController() else {
                return Err(DestructiveConfirmationError::PopoverUnavailable);
            };
            popover.setSourceView(Some(anchor.source_view));
            popover.setSourceRect(anchor.source_rect);
        }

        let callback = OneShot::new(
            on_destructive
                .take()
                .expect("destructive callback is present before UIKit handoff"),
        );
        let action_handler: block2::RcBlock<dyn Fn(std::ptr::NonNull<UIAlertAction>)> =
            block2::RcBlock::new(move |_action| {
                let _ = catch_foreign_panic(|| {
                    callback.invoke();
                });
            });
        let destructive_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&destructive_action_title),
            UIAlertActionStyle::Destructive,
            Some(&action_handler),
            marker,
        );
        let cancel_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&cancel_action_title),
            UIAlertActionStyle::Cancel,
            None,
            marker,
        );
        alert.addAction(&destructive_action);
        alert.addAction(&cancel_action);
        presenter.presentViewController_animated_completion(&alert, true, None);
        Ok(())
    });

    return_callback_on_error(result, on_destructive).map_err(|(error, on_destructive)| {
        DestructiveConfirmationStartError {
            error,
            on_destructive,
        }
    })
}

fn preflight_destructive(
    presenter: &UIViewController,
    anchor: &PopoverAnchor<'_>,
) -> Result<(), DestructiveConfirmationError> {
    if !valid_popover_source_rect(
        anchor.source_rect.origin.x,
        anchor.source_rect.origin.y,
        anchor.source_rect.size.width,
        anchor.source_rect.size.height,
    ) {
        return Err(DestructiveConfirmationError::InvalidPopoverAnchor);
    }
    let presenter_window =
        presenter_window(presenter).map_err(DestructiveConfirmationError::Presenter)?;
    let anchor_window = anchor
        .source_view
        .window()
        .ok_or(DestructiveConfirmationError::AnchorViewNotInWindow)?;
    if !ptr::eq(&*presenter_window, &*anchor_window) {
        return Err(DestructiveConfirmationError::AnchorViewInDifferentWindow);
    }
    Ok(())
}

fn presenter_window(
    presenter: &UIViewController,
) -> Result<objc2::rc::Retained<UIWindow>, PresentationError> {
    if presenter.isBeingPresented() || presenter.isBeingDismissed() {
        return Err(PresentationError::PresenterTransitioning);
    }
    if presenter.presentedViewController().is_some() {
        return Err(PresentationError::AlreadyPresenting);
    }
    let view = presenter
        .viewIfLoaded()
        .ok_or(PresentationError::ViewNotLoaded)?;
    view.window().ok_or(PresentationError::ViewNotInWindow)
}
