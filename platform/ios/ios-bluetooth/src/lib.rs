#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Non-prompting Bluetooth authorization status through the public CoreBluetooth class API."]

mod conversion;
mod queue;

#[cfg(target_os = "ios")]
mod central;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use central::IosBluetoothCentralBackend;

#[cfg(target_os = "ios")]
pub use platform::IosBluetoothBackend;
