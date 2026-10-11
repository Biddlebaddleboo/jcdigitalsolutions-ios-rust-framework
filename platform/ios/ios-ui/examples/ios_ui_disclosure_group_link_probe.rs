#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::IosUiBackend;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let parent_frame = Frame::new(0.0, 0.0, 280.0, 220.0).expect("valid parent frame");
    let expanded_frame = Frame::new(0.0, 0.0, 280.0, 180.0).expect("valid group frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let mut disclosure = match backend.add_disclosure_group(
        &mut parent,
        expanded_frame,
        "Raw Status",
        false,
        |_| {},
    ) {
        Ok(disclosure) => disclosure,
        Err(_) => return,
    };
    let label_frame = Frame::new(8.0, 0.0, 264.0, 120.0).expect("valid content frame");
    let _ = backend.add_label(disclosure.content_view_mut(), label_frame, "{}");
    let _ = disclosure.is_expanded();
    let _ = disclosure.set_expanded(true);
    let _ = disclosure.set_expanded(false);
    core::hint::black_box(disclosure.native_button());
    core::hint::black_box(disclosure.native_stack_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
