#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosScrollView, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let parent_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid parent frame");
    let scroll_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid scroll frame");
    let child_frame = Frame::new(0.0, 0.0, 240.0, 40.0).expect("valid child frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let Ok(mut scroll) = backend.add_scroll_view(&mut parent, scroll_frame) else {
        return;
    };
    let _: &IosScrollView = &scroll;
    let Ok(_label) = backend.add_label(scroll.content_view_mut(), child_frame, "Scroll link probe")
    else {
        return;
    };
    if scroll.set_content_size(240.0, 640.0).is_err() {
        return;
    }
    core::hint::black_box(scroll.native_scroll_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
