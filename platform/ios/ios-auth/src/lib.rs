#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "iOS Local Authentication backend for the portable one-shot local presence contract"]

#[cfg(target_os = "ios")]
mod operation;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
mod tracking;

#[cfg(target_os = "ios")]
pub use platform::{IosAuthenticationBackend, IosAuthenticationFuture};

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use tracking::IosAppTrackingAuthorizationBackend;
