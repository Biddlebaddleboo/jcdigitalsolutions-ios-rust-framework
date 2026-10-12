#[cfg(target_os = "ios")]
use framework_ui::{Frame, ToggleControl, UiBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::{IosToggle, IosUiBackend};

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let mut backend = IosUiBackend::new(main_thread);
    let frame = Frame::new(0.0, 0.0, 51.0, 31.0).expect("valid switch frame");
    let parent_frame = Frame::new(0.0, 0.0, 240.0, 80.0).expect("valid parent frame");
    let Ok(mut parent) = backend.create_view(parent_frame) else {
        return;
    };
    let mut toggle: IosToggle = match backend.add_toggle(&mut parent, frame, false, |_| {}) {
        Ok(toggle) => toggle,
        Err(_) => return,
    };
    let _ = toggle.is_enabled();
    let _ = toggle.set_enabled(false);
    let _ = toggle.set_enabled(true);
    let _ = toggle.is_on();
    let _ = toggle.set_state(true);
    let _ = toggle.set_on(false, true);
    core::hint::black_box(toggle.native_switch());
}

#[cfg(not(target_os = "ios"))]
fn main() {}
