#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS Rust wrapper for single-precision vDSP vector addition."]
#![doc = "\n\nThe iOS API calls Apple's `vDSP_vadd` with unit stride. It is available from iOS 4.0. This crate does not expose other Accelerate APIs, claim Rust parity, or make performance claims. The wrapper has no mutable global state or thread affinity; independent calls may run on separate threads when their buffers do not alias."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{VectorAddError, vector_add};
