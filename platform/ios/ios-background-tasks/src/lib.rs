#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "An opt-in iOS BGTaskScheduler adapter for portable app-refresh tasks."]

#[cfg(target_os = "ios")]
mod platform;

pub use framework_background::{
    AppRefreshBackend, AppRefreshContext, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
    ExpirySignal,
};

#[cfg(target_os = "ios")]
pub use platform::IosBackgroundTasks;
