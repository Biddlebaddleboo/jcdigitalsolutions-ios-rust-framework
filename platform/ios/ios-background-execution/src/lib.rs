#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "UIKit background-execution lease adapter for iOS application targets."]

pub use framework_background_execution::{
    BackgroundExecution, BackgroundExecutionBackend, BackgroundExecutionLease, Error, ErrorKind,
    ExpirySignal, Result,
};

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosBackgroundExecution, IosBackgroundExpiry, IosBackgroundLease};
