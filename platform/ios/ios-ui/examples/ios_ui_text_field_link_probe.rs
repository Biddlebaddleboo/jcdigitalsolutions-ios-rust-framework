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
    let frame = Frame::new(0.0, 0.0, 120.0, 32.0).expect("valid frame");
    let Ok(mut parent) = backend.create_view(frame) else {
        return;
    };
    let Ok(mut field) = backend.add_text_field(&mut parent, frame, Some("A\0😀"), Some("Hint"))
    else {
        return;
    };
    let _ = field.text();
    let _ = field.placeholder();
    let _ = field.set_text(None);
    let _ = field.set_text(Some("replacement"));
    let _ = field.set_placeholder(None);
    let _ = field.set_placeholder(Some("new hint"));
    let _ = field.request_first_responder();
    let _ = field.resign_first_responder();
    let _ = field.native_text_field();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
