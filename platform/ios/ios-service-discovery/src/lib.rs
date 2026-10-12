#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Caller-selected Bonjour discovery through the public Network.framework NWBrowser C API."]

mod state;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosBonjourBrowserBackend, IosBonjourCloseFuture, IosBonjourNextEventFuture,
    IosBonjourStartFuture,
};
