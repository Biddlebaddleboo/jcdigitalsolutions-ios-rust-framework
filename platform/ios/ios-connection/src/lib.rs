#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "TCP streams and listeners through public Network.framework C APIs."]

mod listener_queue;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosCloseFuture, IosConnectFuture, IosConnectionBackend, IosReceiveFuture, IosSendFuture,
    IosTcpListenerAcceptFuture, IosTcpListenerBackend, IosTcpListenerBindFuture,
    IosTcpListenerCloseFuture,
};
