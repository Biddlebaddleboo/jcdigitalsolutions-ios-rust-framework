#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Executor-neutral local notifications through public iOS UserNotifications APIs."]

mod conversion;
mod operation;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosAuthorizationFuture, IosCancelFuture, IosNotificationsBackend, IosPendingRequestCountFuture,
    IosRequestAuthorizationFuture, IosScheduleFuture,
};
