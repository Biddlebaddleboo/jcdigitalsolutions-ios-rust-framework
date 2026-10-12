#![cfg_attr(target_os = "ios", deny(unsafe_op_in_unsafe_fn))]

#[cfg(target_os = "ios")]
fn main() {
    use ios_runtime::main_thread::MainThread;
    use ios_ui::{IosCustomRenderHost, IosCustomRenderTouchInput};
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_ui_kit::UIView;

    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let marker = main_thread.into_objc2();
    let parent = UIView::new(marker);
    let mut host = IosCustomRenderHost::new(marker);
    host.attach_to(
        &parent,
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(64.0, 64.0)),
    );
    let mut input = IosCustomRenderTouchInput::attach(&host, marker);
    let _first_event = input.next_event();
    input.detach();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
