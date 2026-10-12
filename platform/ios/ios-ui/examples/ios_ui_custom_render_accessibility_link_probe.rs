#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{
    AccessibilityActionId, AccessibilityNodeId, IosCustomRenderAccessibility, IosCustomRenderHost,
    SemanticAccessibilityAction, SemanticAccessibilityNode, SemanticFrame,
};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let marker = main_thread.into_objc2();
    let host = IosCustomRenderHost::new(marker);
    let Ok(mut accessibility) = IosCustomRenderAccessibility::new(&host, marker) else {
        return;
    };
    let node = SemanticAccessibilityNode::new(
        AccessibilityNodeId(1),
        SemanticFrame {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        },
    )
    .with_label("Custom renderer action")
    .with_activation(SemanticAccessibilityAction::new(
        AccessibilityActionId(1),
        "Activate",
    ))
    .with_custom_action(SemanticAccessibilityAction::new(
        AccessibilityActionId(2),
        "More options",
    ));
    let _ = accessibility.set_nodes(&[node]);
    let _ = accessibility.request_focus(AccessibilityNodeId(1));
    accessibility.notify_scrolled("Page 1 of 1");
    let _ = accessibility.notify_screen_changed(Some(AccessibilityNodeId(1)));
    let _ = accessibility.take_action();
    core::hint::black_box(&accessibility);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
