#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A status-only Nearby Interaction capability backend for iOS 16 and later."]

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
mod platform;

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
pub use platform::IosNearbyInteractionBackend;
