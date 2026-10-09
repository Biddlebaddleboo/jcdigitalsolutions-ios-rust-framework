#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Non-prompting point-in-time Game Center local-player authentication status for iOS."]

mod platform;

pub use platform::IosGameCenterBackend;
