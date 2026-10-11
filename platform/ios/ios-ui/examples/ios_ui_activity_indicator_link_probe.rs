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
    let frame = Frame::new(0.0, 0.0, 20.0, 20.0).expect("valid frame");
    let Ok(mut parent) = backend.create_view(frame) else {
        return;
    };
    let Ok(mut indicator) = backend.add_activity_indicator(&mut parent, frame, false) else {
        return;
    };
    let _ = indicator.start();
    let _ = indicator.is_animating();
    let _ = indicator.stop();
    let _ = indicator.native_activity_indicator();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
