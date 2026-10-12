#[cfg(target_os = "ios")]
use ios_presentation::{
    AlertAction, AlertActionId, AlertActionStyle, DestructiveConfirmationText, PopoverAnchor,
    present_action_alert, present_destructive_confirmation,
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

    let Some(action_marker) = MainThread::current() else {
        return;
    };
    let actions = [
        AlertAction::new(AlertActionId::new(1), "Keep", AlertActionStyle::Default),
        AlertAction::new(
            AlertActionId::new(2),
            "Delete",
            AlertActionStyle::Destructive,
        ),
        AlertAction::new(AlertActionId::new(3), "Cancel", AlertActionStyle::Cancel),
    ];
    let selected = present_action_alert(
        action_marker,
        &presenter,
        "Choose an action",
        "Select one option.",
        &actions,
        |id| {
            let _ = core::hint::black_box(id.get());
        },
    );
    let _ = core::hint::black_box(selected);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
