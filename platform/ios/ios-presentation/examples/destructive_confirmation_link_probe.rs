#[cfg(target_os = "ios")]
use ios_presentation::{
    DestructiveConfirmationText, PopoverAnchor, present_destructive_confirmation,
};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use objc2_core_foundation::CGRect;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIView, UIViewController};

#[cfg(target_os = "ios")]
fn main() {
    let Some(presenter_marker) = MainThread::current() else {
        return;
    };
    let presenter = UIViewController::new(presenter_marker.into_objc2());
    let Some(anchor_marker) = MainThread::current() else {
        return;
    };
    let anchor_view = UIView::new(anchor_marker.into_objc2());
    let Some(action_marker) = MainThread::current() else {
        return;
    };
    let result = present_destructive_confirmation(
        action_marker,
        &presenter,
        PopoverAnchor::new(&anchor_view, CGRect::ZERO),
        DestructiveConfirmationText::new(
            "Delete item?",
            "This cannot be undone.",
            "Delete",
            "Cancel",
        ),
        || {},
    );
    let _ = core::hint::black_box(result);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
