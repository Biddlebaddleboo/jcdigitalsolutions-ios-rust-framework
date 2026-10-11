#[cfg(target_os = "ios")]
use framework_ui::{Frame, TextFieldInputMode, UiBackend};
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
    let _ = parent.end_editing();
    let _ = field.native_text_field();

    let mut ui = framework_ui::UiClient::new(backend);
    let Ok(mut callback_parent) = ui.create_view(frame) else {
        return;
    };
    let Ok(callback_field) = ui.add_text_field(
        &mut callback_parent,
        frame,
        Some("A\0😀"),
        Some("Hint"),
        |_| {},
    ) else {
        return;
    };
    let _ = callback_field.native_text_field();

    for input_mode in [
        TextFieldInputMode::Text,
        TextFieldInputMode::Password,
        TextFieldInputMode::Number,
        TextFieldInputMode::Decimal,
    ] {
        let Ok(field) = ui.add_text_field_with_mode(
            &mut callback_parent,
            frame,
            input_mode,
            Some("not numeric"),
            Some("Hint"),
            |_| {},
        ) else {
            return;
        };
        let _ = field.native_text_field();
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
