#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let mut lifecycle = ios_app_lifecycle::IosApplicationLifecycle::new(main_thread);
        let _ = lifecycle.state();
        let _ = lifecycle.poll_event();
        if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
            ios_app_lifecycle::open_application_settings(main_thread);
        }
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
