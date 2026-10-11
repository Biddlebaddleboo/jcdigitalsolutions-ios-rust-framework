#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Executor-neutral current-location requests and foreground/background update sessions through public Core Location APIs."]

mod configuration;
mod conversion;
mod operation;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosAccuracyAuthorization, IosAuthorizationFuture, IosBackgroundLocationOptions,
    IosCurrentFuture, IosLocationActivityType, IosLocationBackend, IosLocationFuture,
    IosLocationUpdateFuture, IosLocationUpdates, IosRequestAlwaysAuthorizationFuture,
    IosRequestAuthorizationFuture,
};
