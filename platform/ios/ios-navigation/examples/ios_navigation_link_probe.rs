#[cfg(target_os = "ios")]
use framework_ui::{Frame, UiBackend};
#[cfg(target_os = "ios")]
use ios_navigation::{IosNavigationBarAction, IosNavigationController, IosScreenController};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::IosUiBackend;
#[cfg(target_os = "ios")]
use objc2_ui_kit::UIViewController;

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
    let mut root = IosScreenController::new(main_thread, root_view, Some("Root"));
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Ok(action) = IosNavigationBarAction::new(main_thread, "Settings", Some("gearshape"), || {})
    else {
        return;
    };
    root.set_right_bar_action(Some(&action));
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Ok(replacement) =
        IosNavigationBarAction::new(main_thread, "Refresh", Some("arrow.clockwise"), || {})
    else {
        return;
    };
    root.set_right_bar_action(Some(&replacement));
    drop((action, replacement));
    root.set_right_bar_action(None);
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

    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let native_root = UIViewController::new(main_thread.into_objc2());
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let Ok(mut native_navigation) =
        IosNavigationController::from_native_root(main_thread, &native_root)
    else {
        return;
    };
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let native_screen = UIViewController::new(main_thread.into_objc2());
    let _ = native_navigation.push_view_controller(&native_screen, false);
    drop((native_root, native_screen));
    let _ = native_navigation.native_controller();
    let _ = native_navigation.pop(false);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
