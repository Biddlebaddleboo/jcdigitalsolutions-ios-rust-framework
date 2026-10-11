#[cfg(target_os = "ios")]
use framework_ui::UiBackend;
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosContextMenuAction, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let Ok(parent) = backend
        .create_view(framework_ui::Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid parent frame"))
    else {
        return;
    };
    let Ok(context_menu) = backend.add_context_menu(
        parent.native_view(),
        vec![
            IosContextMenuAction::new(1, "First"),
            IosContextMenuAction::new(2, "Second"),
        ],
        |_| {},
    ) else {
        return;
    };
    core::hint::black_box(context_menu.native_interaction());
    drop(context_menu);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
