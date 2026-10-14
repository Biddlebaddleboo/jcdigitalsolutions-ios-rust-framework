#![cfg_attr(target_os = "ios", deny(unsafe_op_in_unsafe_fn))]

#[cfg(target_os = "ios")]
mod custom_render_accessibility {
    pub use ios_ui::{IosCustomRenderAccessibility, SemanticAccessibilityNode};
}

#[cfg(target_os = "ios")]
mod custom_render_accessibility_policy {
    pub use ios_ui::{AccessibilityActionEvent, SemanticTreeError};
}

#[cfg(target_os = "ios")]
mod custom_render_host {
    pub use ios_ui::{CustomRenderHostError, HostGeometry, IosCustomRenderHost, Rgba8Frame};
}

#[cfg(target_os = "ios")]
mod custom_render_text_input {
    pub use ios_ui::{IosCustomTextInput, IosCustomTextInputSnapshot, IosTextInputTraits};
}

#[cfg(target_os = "ios")]
mod custom_render_touch {
    pub use ios_ui::{IosCustomRenderTouchInput, IosTouchEvent, IosTouchInputFault};
}

#[cfg(target_os = "ios")]
mod display_link {
    pub use ios_ui::{IosDisplayFrame, IosDisplayLink};
}

#[cfg(any(test, target_os = "ios"))]
#[path = "../src/custom_render_session_policy.rs"]
mod custom_render_session_policy;

#[cfg(target_os = "ios")]
#[path = "../src/custom_render_session.rs"]
mod custom_render_session;

#[cfg(target_os = "ios")]
fn main() {
    use custom_render_host::Rgba8Frame;
    use custom_render_session::IosCustomRenderSession;
    use framework_core::Result;
    use ios_runtime::main_thread::MainThread;
    use ios_ui::IosCustomTextInputSnapshot;
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_ui_kit::UIView;

    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let marker = main_thread.into_objc2();
    let root = UIView::new(marker);
    root.setFrame(CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(64.0, 64.0)));
    let mut session = IosCustomRenderSession::new(root, marker);
    let _ = session.root_view().bounds();
    let _ = session.native_render_view().frame();
    let _ = session.set_frame(CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(64.0, 64.0)));
    let _geometry = session.geometry();
    let pixels = [0_u8; 16];
    let frame = Rgba8Frame::new(2, 2, 8, &pixels, None).unwrap();
    let _ = session.present(&frame);
    let _ = session.enable_touch_input();
    let _ = session.next_touch_event();
    let _ = session.touch_fault();
    let _ = session.enable_accessibility();
    let _ = session.set_accessibility_nodes(&[]);
    let _ = session.take_accessibility_action();
    let _ = session.accessibility_mut();
    let _ = session.enable_text_input(
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(64.0, 44.0)),
        "",
        ios_ui::IosTextInputTraits::default(),
        |_snapshot: Result<IosCustomTextInputSnapshot>| {},
    );
    let _ = session.enable_display_link(|_| {});
    let _ = session.text_input();
    let _ = session.text_input_mut();
    let _ = session.is_active();
    let _ = session.set_active(false);
    let _ = session.set_active(true);
    session.detach();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
