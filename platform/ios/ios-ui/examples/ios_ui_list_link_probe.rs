#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosListRow, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid list frame");
    let Ok(mut parent) = backend.create_view(frame) else {
        return;
    };
    let rows = vec![
        IosListRow::new(101, "first"),
        IosListRow::new(202, "second").with_detail("secondary"),
    ];
    let Ok(mut list) = backend.add_list_view(&mut parent, frame, rows, |_| {}) else {
        return;
    };
    let _ = list.replace_rows(vec![IosListRow::new(303, "updated")]);
    let Ok(mut copy_button) = backend.add_button(
        &mut parent,
        Frame::new(0.0, 0.0, 44.0, 44.0).expect("valid button frame"),
        "Copy",
        || {},
    ) else {
        return;
    };
    let _ = copy_button.set_accessibility_label(Some("Copy\0 log entry 🚗"));
    let _ = copy_button.set_accessibility_hint(Some("Copies the entry to the clipboard"));
    let _ = copy_button.set_accessibility_identifier(Some("log.copy"));
    let _ = copy_button.set_accessibility_label(Some(""));
    let _ = copy_button.set_accessibility_hint(None);
    let _ = copy_button.set_accessibility_identifier(None);
    core::hint::black_box(list.native_table_view());
    core::hint::black_box(copy_button.native_button());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
