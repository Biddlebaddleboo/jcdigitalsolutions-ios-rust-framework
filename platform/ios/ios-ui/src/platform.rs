use core::cell::RefCell;
use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use framework_ui::{ButtonControl, Frame, LabelControl, UiBackend};
use ios_runtime::ffi::catch_unwind;
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_ui_kit::{UIButton, UIControlEvents, UIControlState, UILabel, UIView};

/// A UIKit backend for caller-owned native controls.
///
/// Construct it on the main thread. It owns no window, controller, scene, or global service.
pub struct IosUiBackend {
    _main_thread: MainThread,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit container view.
///
/// The caller owns its hierarchy placement and lifetime. Use [`Self::native_view`] to attach it
/// to a caller-owned `UIViewController` or another native parent.
pub struct IosView {
    native: Retained<UIView>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit text label.
pub struct IosLabel {
    native: Retained<UILabel>,
    _not_send: PhantomData<Rc<()>>,
}

/// A retained UIKit button and its Rust target/action callback owner.
///
/// Keep this handle alive while the button action should remain active. The native parent view may
/// keep the visual `UIButton` alive after this handle drops, but this handle is the Rust owner of
/// the callback target.
pub struct IosButton {
    native: Retained<UIButton>,
    _target: Retained<ButtonTarget>,
    _not_send: PhantomData<Rc<()>>,
}

struct ButtonTargetState {
    action: RefCell<Box<dyn FnMut()>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ButtonTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiButtonTarget"]
    #[ivars = ButtonTargetState]
    struct ButtonTarget;

    impl ButtonTarget {
        #[unsafe(method(buttonTapped:))]
        fn button_tapped(&self, _sender: &UIButton) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| {
                if let Ok(mut action) = self.ivars().action.try_borrow_mut() {
                    action();
                }
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for ButtonTarget {}
);

impl ButtonTarget {
    fn with_action(mtm: MainThreadMarker, action: Box<dyn FnMut()>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ButtonTargetState {
            action: RefCell::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

impl IosUiBackend {
    /// Creates a backend from a typed proof that the current call is on UIKit's main thread.
    pub fn new(main_thread: MainThread) -> Self {
        Self {
            _main_thread: main_thread,
            _not_send: PhantomData,
        }
    }
}

impl IosView {
    /// Borrows the native container view for UIKit integration.
    pub fn native_view(&self) -> &UIView {
        &self.native
    }
}

impl IosLabel {
    /// Borrows the native label for UIKit integration.
    pub fn native_label(&self) -> &UILabel {
        &self.native
    }
}

impl IosButton {
    /// Borrows the native button for UIKit integration.
    pub fn native_button(&self) -> &UIButton {
        &self.native
    }
}

impl Drop for IosButton {
    fn drop(&mut self) {
        // SAFETY: the button handle is main-thread-only, and the target is the exact retained
        // receiver registered for this selector and event. Remove only this action before release.
        unsafe {
            self.native.removeTarget_action_forControlEvents(
                Some(&*self._target),
                Some(sel!(buttonTapped:)),
                UIControlEvents::TouchUpInside,
            );
        }
    }
}

impl LabelControl for IosLabel {
    fn set_text(&mut self, text: &str) -> Result<()> {
        let _marker = main_thread_marker()?;
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            self.native.setText(Some(&text));
        });
        Ok(())
    }
}

impl ButtonControl for IosButton {
    fn set_title(&mut self, title: &str) -> Result<()> {
        let _marker = main_thread_marker()?;
        autoreleasepool(|_| {
            let title = NSString::from_str(title);
            self.native
                .setTitle_forState(Some(&title), UIControlState::Normal);
        });
        Ok(())
    }
}

impl UiBackend for IosUiBackend {
    type View = IosView;
    type Label = IosLabel;
    type Button = IosButton;

    fn create_view(&mut self, frame: Frame) -> Result<Self::View> {
        let marker = main_thread_marker()?;
        let native = UIView::new(marker);
        native.setFrame(native_frame(frame));
        Ok(IosView {
            native,
            _not_send: PhantomData,
        })
    }

    fn add_label(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        text: &str,
    ) -> Result<Self::Label> {
        let marker = main_thread_marker()?;
        let native = UILabel::new(marker);
        native.setFrame(native_frame(frame));
        autoreleasepool(|_| {
            let text = NSString::from_str(text);
            native.setText(Some(&text));
        });
        parent.native.addSubview(&native);
        Ok(IosLabel {
            native,
            _not_send: PhantomData,
        })
    }

    fn add_button<F>(
        &mut self,
        parent: &mut Self::View,
        frame: Frame,
        title: &str,
        action: F,
    ) -> Result<Self::Button>
    where
        F: FnMut() + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UIButton::new(marker);
        native.setFrame(native_frame(frame));
        autoreleasepool(|_| {
            let title = NSString::from_str(title);
            native.setTitle_forState(Some(&title), UIControlState::Normal);
        });
        let target = ButtonTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is strongly retained by the returned `IosButton`, and the target
        // implements the exact `buttonTapped:` selector registered for `TouchUpInside`.
        unsafe {
            native.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(buttonTapped:),
                UIControlEvents::TouchUpInside,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosButton {
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }
}

fn main_thread_marker() -> Result<MainThreadMarker> {
    MainThread::current()
        .map(MainThread::into_objc2)
        .ok_or_else(|| Error::new(ErrorKind::Unavailable))
}

fn native_frame(frame: Frame) -> CGRect {
    CGRect::new(
        CGPoint::new(frame.x(), frame.y()),
        CGSize::new(frame.width(), frame.height()),
    )
}
