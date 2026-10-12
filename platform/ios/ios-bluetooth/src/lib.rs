#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Bluetooth authorization, foreground central discovery, explicit connections, and bounded GATT operations."]

mod characteristic_notification_policy;
mod characteristic_read_policy;
mod characteristic_read_selection;
mod characteristic_write_policy;
mod connection_policy;
mod conversion;
mod peripheral_policy;
mod queue;
mod service_discovery_policy;

#[cfg(target_os = "ios")]
mod central;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
mod peripheral;

#[cfg(target_os = "ios")]
pub use central::IosBluetoothCentralBackend;

#[cfg(target_os = "ios")]
pub use peripheral::IosBluetoothPeripheralBackend;

#[cfg(target_os = "ios")]
pub use platform::IosBluetoothBackend;
