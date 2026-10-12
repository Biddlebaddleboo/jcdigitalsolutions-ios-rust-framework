use core::marker::PhantomData;
use core::ptr::NonNull;
use core::result::Result;
use std::rc::Rc;

use block2::RcBlock;
use framework_core::{Error, ErrorKind};
use ios_runtime::main_thread::MainThread;
use ios_ui::IosView;
use objc2::MainThreadOnly;
use objc2::rc::{Retained, autoreleasepool};
use objc2_foundation::NSString;
use objc2_ui_kit::{UIAction, UIBarButtonItem, UIImage, UINavigationController, UIViewController};

use crate::navigation_action_policy::{
    SharedActionCallback, callback_cell, invoke_action_callback, validate_action_input,
};
use crate::push_policy::perform_push_if_unparented;

/// One generic native screen controller backed by an owned `IosView`.
///
/// UIKit owns the view after `new` returns. The screen and its UIKit objects are main-thread-only
/// and never `Send` or `Sync`.
pub struct IosScreenController {
    native: Retained<UIViewController>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit primary action for the right side of a navigation bar.
///
/// The required title is also the action's accessible name. The optional symbol must exist in the
/// system symbol catalog on the current OS. This API requires iOS 14.0 and returns
/// [`ErrorKind::Unavailable`] on older iOS versions. It does not change the crate's iOS 12 package
/// floor. The action and its callback are main-thread-only and never `Send` or `Sync`.
pub struct IosNavigationBarAction {
    native: Retained<UIBarButtonItem>,
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

    /// Installs, replaces, or clears the screen's right navigation-bar action.
    ///
    /// UIKit retains the item and its copied primary action. The callback remains alive if the
    /// borrowed [`IosNavigationBarAction`] value is dropped after installation. Pass `None` to
    /// remove the item. This changes only the right-side item; screen composition and route policy
    /// remain app-owned.
    pub fn set_right_bar_action(&mut self, action: Option<&IosNavigationBarAction>) {
        self.native
            .navigationItem()
            .setRightBarButtonItem(action.map(|action| &*action.native));
    }
}

impl IosNavigationBarAction {
    /// Creates a main-thread UIKit action with an accessible title and optional SF-symbol name.
    ///
    /// The title must contain a non-whitespace character. An empty, whitespace-only, or
    /// unsupported system-symbol name returns `InvalidInput` or `Unavailable` before attachment.
    /// The supplied `FnMut()` callback is retained by UIKit, panic-contained, and ignored during a
    /// reentrant invocation. The iOS 14 runtime availability check is explicit; the package
    /// retains its iOS 12 deployment floor.
    pub fn new<F>(
        main_thread: MainThread,
        title: &str,
        system_symbol: Option<&str>,
        callback: F,
    ) -> Result<Self, Error>
    where
        F: FnMut() + 'static,
    {
        validate_action_input(title, system_symbol)?;
        if !objc2::available!(ios = 14.0, ..) {
            return Err(Error::new(ErrorKind::Unavailable));
        }

        let marker = main_thread.into_objc2();
        autoreleasepool(|_| {
            let title = NSString::from_str(title);
            let image = match system_symbol {
                Some(name) => {
                    let name = NSString::from_str(name);
                    Some(
                        UIImage::systemImageNamed(&name)
                            .ok_or_else(|| Error::new(ErrorKind::Unavailable))?,
                    )
                }
                None => None,
            };
            let callback: SharedActionCallback = callback_cell(callback);
            let handler_callback = callback.clone();
            let handler: RcBlock<dyn Fn(NonNull<UIAction>)> = RcBlock::new(move |_action| {
                invoke_action_callback(&handler_callback);
            });
            // SAFETY: `handler` remains live through this call, is a valid block pointer, and
            // UIAction copies and retains its handler. The iOS 14 guard precedes both this
            // iOS 13+ action API and the UIBarButtonItem primary-action initializer.
            let native_action = unsafe {
                UIAction::actionWithTitle_image_identifier_handler(
                    &title,
                    image.as_deref(),
                    None,
                    RcBlock::as_ptr(&handler),
                    marker,
                )
            };
            // This public UIKit initializer copies the retained primary action, which owns its
            // callback block.
            let native = UIBarButtonItem::initWithPrimaryAction(
                UIBarButtonItem::alloc(marker),
                Some(&native_action),
            );
            Ok(Self {
                native,
                _not_send: PhantomData,
            })
        })
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
