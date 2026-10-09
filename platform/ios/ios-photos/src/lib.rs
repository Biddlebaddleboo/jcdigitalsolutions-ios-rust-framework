#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "iOS PhotoKit read/write authorization backend for `framework-photos`"]

mod operation;
mod status;

#[cfg(target_os = "ios")]
mod platform;

pub use status::authorization_status_from_native;

#[cfg(target_os = "ios")]
pub use platform::{IosPhotosAuthorizationFuture, IosPhotosBackend};
