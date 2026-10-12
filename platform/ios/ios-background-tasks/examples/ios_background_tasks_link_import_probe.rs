#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_background::{
    AppRefreshBackend, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
    ExternalPowerRequirement, NetworkConnectivityRequirement, ProcessingTaskBackend,
    ProcessingTaskId, ProcessingTaskOutcome, ProcessingTaskRequest, ProcessingTaskRequirements,
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

    let processing_id = ProcessingTaskId::new("com.example.app.processing").unwrap();
    let requirements = ProcessingTaskRequirements::new()
        .with_network_connectivity(NetworkConnectivityRequirement::Required)
        .with_external_power(ExternalPowerRequirement::Required);
    let processing_request = ProcessingTaskRequest::new(processing_id.clone(), requirements);
    let _ = core::hint::black_box(backend.register_processing_task(&processing_id, |task| {
        if task.is_expired() {
            ProcessingTaskOutcome::Failed
        } else {
            ProcessingTaskOutcome::Succeeded
        }
    }));
    let _ = core::hint::black_box(backend.submit_processing_task(&processing_request));
    let _ = core::hint::black_box(backend.cancel_processing_task(&processing_id));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
