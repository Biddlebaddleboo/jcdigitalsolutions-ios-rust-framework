#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_ui::IosUiBackend;

#[cfg(target_os = "ios")]
fn main() {
    let Some(main_thread) = MainThread::current() else {
        return;
    };
    let backend = IosUiBackend::new(main_thread);
    let _handled = backend.send_resign_first_responder_action();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
