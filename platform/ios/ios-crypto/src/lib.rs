#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A narrow iOS-only Rust wrapper around Apple's CommonCrypto SHA-256 C API."]
#![doc = "\n\nThis crate exposes only one-shot SHA-256 through `CC_SHA256`. It adds no portable `framework-crypto` contract, does not replace or default over an Apple implementation, and makes no parity, certification, or performance claim. It performs no key management, secure storage, or other cryptographic operation."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{Sha256Error, sha256};
