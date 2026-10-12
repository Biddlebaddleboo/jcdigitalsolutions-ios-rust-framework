#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Bluetooth authorization, foreground central discovery, and explicit central connections."]

mod connection_policy;
mod conversion;
mod queue;
mod service_discovery_policy;

#[cfg(target_os = "ios")]
mod central;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use central::IosBluetoothCentralBackend;

#[cfg(target_os = "ios")]
pub use platform::IosBluetoothBackend;
