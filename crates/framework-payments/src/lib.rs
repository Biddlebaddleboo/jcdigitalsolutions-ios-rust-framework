#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Platform-exclusive payment capability queries"]

/// iOS-only Apple Pay capability status
#[cfg(target_os = "ios")]
pub mod ios;
