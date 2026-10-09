#[cfg(target_os = "ios")]
fn main() {
    if let Some(main_thread) = ios_runtime::main_thread::MainThread::current() {
        let _ = ios_app_scenes::snapshot(main_thread);
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
