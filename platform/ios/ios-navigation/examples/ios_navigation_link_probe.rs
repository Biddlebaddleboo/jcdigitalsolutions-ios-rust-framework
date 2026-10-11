#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_navigation::{IosNavigationController, IosScreenController};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::IosUiBackend;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut ui = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 320.0, 480.0).expect("valid frame");
    let Ok(root_view) = ui.create_view(frame) else {
        return;
    };
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let root = IosScreenController::new(main_thread, root_view, Some("Root"));
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut navigation = IosNavigationController::new(main_thread, root);
    let Ok(next_view) = ui.create_view(frame) else {
        return;
    };
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let next = IosScreenController::new(main_thread, next_view, Some("Next"));
    let _ = navigation.push(&next, false);
    let _ = navigation.native_controller();
    let _ = navigation.pop(false);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
