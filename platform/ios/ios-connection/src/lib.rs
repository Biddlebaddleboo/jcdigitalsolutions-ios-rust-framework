#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Outbound secure TCP byte streams through public Network.framework C APIs."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosCloseFuture, IosConnectFuture, IosConnectionBackend, IosReceiveFuture, IosSendFuture,
};
