#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS-only Metal Performance Shaders preferred-device presence query."]
#![doc = "\n\nThis crate exposes only whether `MPSGetPreferredDevice` returns a device with default options. It does not expose a native handle, submit GPU work, or claim that any MPS operation or workload is supported. The query requires an iOS deployment target of 12.2 or later. It has no portable facade, permissions, Info.plist key, or entitlement requirement."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::preferred_mps_device_available;
