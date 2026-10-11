#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "A small native UIKit screen-navigation backend for iOS."]

#[cfg(any(test, target_os = "ios"))]
mod push_policy;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosNavigationController, IosScreenController};
