use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_ui_kit::{UIViewController, UIWindow, UIWindowScene};

use crate::validate_root_attachment;

/// A UIKit window attached to a caller-owned scene.
///
/// The host supplies a live scene and root controller. This handle owns the window, but does not
/// configure the app or scene delegate. The value is main-thread-bound and is neither `Send` nor
/// `Sync`.
#[must_use = "dropping the handle hides its window and detaches its root controller"]
pub struct IosSceneWindow {
    window: Retained<UIWindow>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosSceneWindow {
    /// Creates a window for the caller's live `scene`, attaches the unparented `root`, and asks
    /// UIKit to show it.
    ///
    /// Returns `framework_core::ErrorKind::InvalidInput` if `root` already has a parent controller
    /// or if its already-loaded view belongs to a window. The check does not load the view or
    /// reparent the controller. A successful return records the requested UIKit calls; it does not
    /// prove that the window became visible or key.
    pub fn new(
        main_thread: MainThread,
        scene: &UIWindowScene,
        root: &UIViewController,
    ) -> Result<Self> {
        validate_root_attachment(root.parentViewController().is_some(), || {
            root.isViewLoaded()
                && root
                    .viewIfLoaded()
                    .is_some_and(|view| view.window().is_some())
        })?;

        let window =
            UIWindow::initWithWindowScene(UIWindow::alloc(main_thread.into_objc2()), scene);
        window.setRootViewController(Some(root));
        window.makeKeyAndVisible();
        Ok(Self {
            window,
            _not_send: PhantomData,
        })
    }

    /// Borrows the native window for app-owned UIKit integration.
    pub fn native_window(&self) -> &UIWindow {
        &self.window
    }
}

impl Drop for IosSceneWindow {
    fn drop(&mut self) {
        self.window.setHidden(true);
        self.window.setRootViewController(None);
    }
}
