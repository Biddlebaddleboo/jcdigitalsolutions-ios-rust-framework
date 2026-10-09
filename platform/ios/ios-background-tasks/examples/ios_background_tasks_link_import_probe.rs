#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_background::{
    AppRefreshBackend, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
};
#[cfg(target_os = "ios")]
use ios_background_tasks::IosBackgroundTasks;

#[cfg(target_os = "ios")]
fn main() {
    let backend = IosBackgroundTasks::new();
    let task_id = AppRefreshTaskId::new("com.example.app.refresh").unwrap();
    let request = AppRefreshRequest::new(task_id.clone());
    let _ = core::hint::black_box(
        backend.register_app_refresh(&task_id, |_| AppRefreshOutcome::Failed),
    );
    let _ = core::hint::black_box(backend.submit_app_refresh(&request));
    let _ = core::hint::black_box(backend.cancel_app_refresh(&task_id));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
