use std::cell::{Cell, OnceCell};

use ios_runtime::{ffi::catch_unwind, main_thread::MainThread};
use objc2::rc::{Allocated, Retained};
use objc2::{ClassType, DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UIButton, UIColor, UIControlEvents, UIControlState,
    UILabel, UIScreen, UIView, UIViewController, UIWindow,
};

#[derive(Default)]
struct AppState {
    window: OnceCell<Retained<UIWindow>>,
    target: OnceCell<Retained<ButtonTarget>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - AppDelegate does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMinimalAppDelegate"]
    #[ivars = AppState]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(AppState::default());
            // SAFETY: `this` is a fresh allocation of this NSObject subclass and NSObject's `init` is valid.
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(application:didFinishLaunchingWithOptions:))]
        unsafe fn did_finish_launching(
            &self,
            _application: &UIApplication,
            _options: Option<&objc2_foundation::NSDictionary>,
        ) -> bool {
            catch_unwind(|| self.install_ui()).unwrap_or(false)
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and the class uses NSObject's implementation.
    unsafe impl NSObjectProtocol for AppDelegate {}
    // SAFETY: the class implements the documented launch selector with its generated ABI signature.
    unsafe impl UIApplicationDelegate for AppDelegate {}
);

struct TargetState {
    label: Retained<UILabel>,
    taps: Cell<u32>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - ButtonTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMinimalButtonTarget"]
    #[ivars = TargetState]
    struct ButtonTarget;

    impl ButtonTarget {
        #[unsafe(method(buttonTapped:))]
        fn button_tapped(&self, _sender: &UIButton) {
            let _ = catch_unwind(|| {
                let taps = self.ivars().taps.get().wrapping_add(1);
                self.ivars().taps.set(taps);
                let text = NSString::from_str(&format!("Rust callback: {taps} tap(s)"));
                self.ivars().label.setText(Some(&text));
            });
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and the class uses NSObject's implementation.
    unsafe impl NSObjectProtocol for ButtonTarget {}
);

impl ButtonTarget {
    fn with_label(mtm: objc2::MainThreadMarker, label: Retained<UILabel>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(TargetState {
            label,
            taps: Cell::new(0),
        });
        // SAFETY: `this` is a fresh allocation of this NSObject subclass and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

impl AppDelegate {
    #[allow(deprecated)]
    fn install_ui(&self) -> bool {
        let mtm = MainThread::current()
            .expect("UIKit launch callback must be on the main thread")
            .into_objc2();
        let bounds = UIScreen::mainScreen(mtm).bounds();
        let label_frame = CGRect::new(
            CGPoint::new(24.0, 96.0),
            CGSize::new(bounds.size.width - 48.0, 44.0),
        );
        let button_frame = CGRect::new(
            CGPoint::new(24.0, 152.0),
            CGSize::new(bounds.size.width - 48.0, 52.0),
        );

        let window = UIWindow::initWithFrame(UIWindow::alloc(mtm), bounds);
        let controller = UIViewController::new(mtm);
        let root_view = UIView::new(mtm);
        root_view.setFrame(bounds);
        root_view.setBackgroundColor(Some(&UIColor::systemBackgroundColor()));

        let label = UILabel::new(mtm);
        label.setFrame(label_frame);
        // SAFETY: a non-null adaptive UIKit system color is valid for UILabel text.
        unsafe {
            label.setTextColor(Some(&UIColor::labelColor()));
        }
        let initial_text = NSString::from_str("Rust created this UIKit screen");
        label.setText(Some(&initial_text));

        let button = UIButton::new(mtm);
        button.setFrame(button_frame);
        let title = NSString::from_str("Call Rust");
        button.setTitle_forState(Some(&title), UIControlState::Normal);
        button.setTitleColor_forState(Some(&UIColor::systemBlueColor()), UIControlState::Normal);

        let target = ButtonTarget::with_label(mtm, label.clone());
        // SAFETY: `target` is the retained Rust target object and implements the exact `buttonTapped:` selector.
        unsafe {
            button.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(buttonTapped:),
                UIControlEvents::TouchUpInside,
            );
        }

        root_view.addSubview(&label);
        root_view.addSubview(&button);
        controller.setView(Some(&root_view));
        window.setRootViewController(Some(&controller));

        if self.ivars().target.set(target).is_err() || self.ivars().window.set(window).is_err() {
            return false;
        }
        self.ivars()
            .window
            .get()
            .expect("window retained")
            .makeKeyAndVisible();
        if std::env::args_os().any(|arg| arg == "--exercise-rust-button-callback") {
            button.sendActionsForControlEvents(UIControlEvents::TouchUpInside);
        }
        true
    }
}

pub fn run() {
    let mtm = MainThread::current()
        .expect("UIApplication::main must start on the main thread")
        .into_objc2();
    let delegate_name = NSString::from_class(AppDelegate::class());
    UIApplication::main(None, Some(&delegate_name), mtm);
}
