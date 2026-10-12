#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Core NFC reader support and one-shot NDEF reads for iOS."]

#[cfg(any(test, all(target_os = "ios", feature = "ndef-read")))]
extern crate std;

#[cfg(any(test, all(target_os = "ios", feature = "ndef-read")))]
mod read_state;

#[cfg(all(target_os = "ios", feature = "ndef-read"))]
mod ndef;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::CoreNfcReaderBackend;

#[cfg(all(target_os = "ios", feature = "ndef-read"))]
pub use ndef::CoreNfcNdefBackend;
