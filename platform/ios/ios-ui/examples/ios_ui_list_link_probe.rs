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
    core::hint::black_box(list.native_table_view());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
