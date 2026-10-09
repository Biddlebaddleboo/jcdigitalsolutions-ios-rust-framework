#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "One-shot raw accelerometer samples through public Core Motion APIs."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosCurrentAccelerationFuture, IosMotionBackend};

/// The Core Motion manager type accepted by [`IosMotionBackend`].
#[cfg(target_os = "ios")]
pub use objc2_core_motion::CMMotionManager;

/// The Objective-C main-thread marker required by [`IosMotionBackend`].
#[cfg(target_os = "ios")]
pub use objc2::MainThreadMarker;
