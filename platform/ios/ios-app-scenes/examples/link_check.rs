#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let _ = ios_app_scenes::snapshot(main_thread);
    }
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let mut events = ios_app_scenes::IosSceneLifecycleEvents::new(main_thread);
        let _ = events.poll_event();
        let _ = events.dropped_event_count();
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
