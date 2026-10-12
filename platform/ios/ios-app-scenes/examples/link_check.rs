#[cfg(target_os = "ios")]
fn link_scene_window(
    main_thread: ios_runtime::main_thread::MainThread,
    scene: &objc2_ui_kit::UIWindowScene,
    root: &objc2_ui_kit::UIViewController,
) {
    if let Ok(window) = ios_app_scenes::IosSceneWindow::new(main_thread, scene, root) {
        let _ = window.native_window().isKeyWindow();
        drop(window);
    }
}

#[cfg(target_os = "ios")]
fn link_scene_activation_snapshot(main_thread: ios_runtime::main_thread::MainThread) {
    for scene in ios_app_scenes::scene_activation_snapshot(main_thread) {
        let _ = scene.session_identifier().as_utf16_code_units();
        let _ = scene.state();
    }
}

#[cfg(target_os = "ios")]
#[used]
static SCENE_ACTIVATION_SNAPSHOT_LINK: fn(ios_runtime::main_thread::MainThread) =
    link_scene_activation_snapshot;

#[cfg(target_os = "ios")]
#[used]
static SCENE_WINDOW_LINK: fn(
    ios_runtime::main_thread::MainThread,
    &objc2_ui_kit::UIWindowScene,
    &objc2_ui_kit::UIViewController,
) = link_scene_window;

#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let _ = ios_app_scenes::snapshot(main_thread);
    }
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        link_scene_activation_snapshot(main_thread);
    }
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let mut events = ios_app_scenes::IosSceneLifecycleEvents::new(main_thread);
        events.set_event_handler(|event| {
            let _ = event.kind();
        });
        let _ = events.poll_event();
        let _ = events.dropped_event_count();
        let _ = events.dropped_handler_event_count();
        events.clear_event_handler();
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
