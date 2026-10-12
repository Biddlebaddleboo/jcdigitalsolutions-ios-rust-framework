#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "An opt-in iOS BGTaskScheduler adapter for portable refresh and processing tasks."]

#[cfg(any(target_os = "ios", test))]
mod lifecycle;
#[cfg(target_os = "ios")]
mod platform;

pub use framework_background::{
    AppRefreshBackend, AppRefreshContext, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
    ExpirySignal, ExternalPowerRequirement, NetworkConnectivityRequirement, ProcessingTaskBackend,
    ProcessingTaskContext, ProcessingTaskId, ProcessingTaskOutcome, ProcessingTaskRequest,
    ProcessingTaskRequirements,
};

#[cfg(target_os = "ios")]
pub use platform::IosBackgroundTasks;
