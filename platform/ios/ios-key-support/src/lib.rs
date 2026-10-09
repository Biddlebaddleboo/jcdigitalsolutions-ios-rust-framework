#![cfg(target_os = "ios")]
#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "iOS Security backend for P-256 ECDSA/SHA-256 verification suitability only."]

mod p256_support;

pub use p256_support::IosP256VerificationSupport;
