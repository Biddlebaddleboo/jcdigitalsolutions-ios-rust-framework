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
    let Ok(mut parent) =
        backend.create_view(Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid parent frame"))
    else {
        return;
    };
    let Ok(mut text_view) = backend.add_selectable_text_view(
        &mut parent,
        Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid text frame"),
        "first\0line\nsecond",
    ) else {
        return;
    };
    core::hint::black_box(text_view.native_text_view());
    let _ = core::hint::black_box(text_view.text());
    let _ = text_view.set_text("updated\ntext");
    let _ = text_view.set_text("");
}

#[cfg(not(target_os = "ios"))]
fn main() {}
