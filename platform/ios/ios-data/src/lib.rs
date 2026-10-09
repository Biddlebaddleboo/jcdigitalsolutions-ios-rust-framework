#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Explicit copies between portable framework bytes and immutable Core Foundation data."]

extern crate alloc;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosData, IosDataError};
