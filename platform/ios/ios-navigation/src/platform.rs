use core::marker::PhantomData;
use core::result::Result;
use std::rc::Rc;

use framework_core::Error;
use ios_runtime::main_thread::MainThread;
use ios_ui::IosView;
use objc2::MainThreadOnly;
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::NSString;
use objc2_ui_kit::{UINavigationController, UIViewController};

use crate::push_policy::perform_push_if_unparented;

/// One generic native screen controller backed by an owned `IosView`.
///
/// UIKit owns the view after `new` returns. The screen and its UIKit objects are main-thread-only
/// and never `Send` or `Sync`.
pub struct IosScreenController {
    native: Retained<UIViewController>,
    _not_send: PhantomData<Rc<()>>,
}

/// A native UIKit navigation controller whose route stack remains UIKit-owned.
///
/// The application owns its scene and window and may attach [`Self::native_controller`] there.
pub struct IosNavigationController {
    native: Retained<UINavigationController>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosScreenController {
    /// Creates a generic screen controller from a completed view and optional navigation title.
    ///
    /// The view handle is consumed. `UIViewController` retains the native view; this wrapper does
    /// not keep a second view handle. Pass `None` to clear the navigation title or `Some("")` for
    /// an explicitly empty title.
    pub fn new(main_thread: MainThread, view: IosView, title: Option<&str>) -> Self {
        let native = UIViewController::new(main_thread.into_objc2());
        native.setView(Some(view.native_view()));
        autoreleasepool(|_| {
            let title = title.map(NSString::from_str);
            native.setTitle(title.as_deref());
        });
        Self {
            native,
            _not_send: PhantomData,
        }
    }

    /// Replaces or clears the navigation title with an explicit-length UTF-8 value.
    pub fn set_title(&mut self, title: Option<&str>) {
        autoreleasepool(|_| {
            let title = title.map(NSString::from_str);
            self.native.setTitle(title.as_deref());
        });
    }
}

impl IosNavigationController {
    /// Creates a navigation controller with `root` as its initial screen.
    ///
    /// UIKit retains the root controller and owns the stack. This value owns no scene or window.
    pub fn new(main_thread: MainThread, root: IosScreenController) -> Self {
        let native = UINavigationController::initWithRootViewController(
            UINavigationController::alloc(main_thread.into_objc2()),
            &root.native,
        );
        Self {
            native,
            _not_send: PhantomData,
        }
    }

    /// Pushes an unparented screen through UIKit.
    ///
    /// Returns `framework_core::ErrorKind::InvalidInput` if the controller already belongs to a
    /// parent controller.
    /// UIKit owns the pushed controller after this call. Its own transition and back-button
    /// behavior remain authoritative; `animated` is passed through unchanged.
    pub fn push(&mut self, screen: &IosScreenController, animated: bool) -> Result<(), Error> {
        perform_push_if_unparented(screen.native.parentViewController().is_some(), || {
            self.native
                .pushViewController_animated(&screen.native, animated);
        })
        .map_err(Error::new)?;
        Ok(())
    }

    /// Pops the top screen through UIKit and returns whether a controller was removed.
    ///
    /// `false` means UIKit returned no popped controller, such as when the root is already topmost.
    /// `animated` is passed through unchanged. A `true` result does not prove that the transition
    /// completed or that a screen was visibly displayed.
    pub fn pop(&mut self, animated: bool) -> bool {
        self.native.popViewControllerAnimated(animated).is_some()
    }

    /// Borrows the native navigation controller for app-owned scene or window attachment.
    pub fn native_controller(&self) -> &UINavigationController {
        &self.native
    }
}
