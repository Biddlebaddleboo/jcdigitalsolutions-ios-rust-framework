#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Capability-scoped Swift object ownership. No Swift ABI runtime is linked unless `apple-runtime` is enabled."]

#[cfg(feature = "apple-runtime")]
mod retained;

#[cfg(feature = "apple-runtime")]
pub use retained::{SwiftClass, SwiftObject, SwiftRetained};
