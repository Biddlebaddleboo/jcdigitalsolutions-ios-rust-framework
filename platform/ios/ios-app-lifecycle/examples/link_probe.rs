#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let mut lifecycle = ios_app_lifecycle::IosApplicationLifecycle::new(main_thread);
        let _ = lifecycle.state();
        let _ = lifecycle.poll_event();
        if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
            ios_app_lifecycle::open_application_settings(main_thread);
        }
        if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
            let _ = ios_app_lifecycle::open_application_uri(
                main_thread,
                "sample+app://open?item=1",
                |_| {},
            );
        }
        #[cfg(feature = "main-thread-dispatch")]
        let _ = ios_runtime::dispatch::run_on_main(|main_thread| {
            let _ = main_thread.into_objc2();
        });
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
