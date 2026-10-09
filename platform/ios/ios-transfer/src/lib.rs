#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Durable iOS background HTTP file downloads through Foundation URLSession."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
mod record;

#[cfg(target_os = "ios")]
pub use platform::{BackgroundEventError, IosTransferBackend};
