#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "An opt-in local notification response bridge for iOS"]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{InstallError, IosNotificationResponses};
