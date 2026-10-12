#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Core NFC reader support and one-shot NDEF reads and writes for iOS."]

#[cfg(any(test, all(target_os = "ios", feature = "ndef-read")))]
extern crate std;

#[cfg(any(test, all(target_os = "ios", feature = "ndef-read")))]
mod read_state;

#[cfg(any(test, all(target_os = "ios", feature = "ndef-write")))]
mod write_state;

#[cfg(all(target_os = "ios", feature = "ndef-read"))]
mod ndef;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::CoreNfcReaderBackend;

#[cfg(all(target_os = "ios", feature = "ndef-read"))]
pub use ndef::CoreNfcNdefBackend;

#[cfg(all(target_os = "ios", feature = "ndef-write"))]
mod ndef_write;

#[cfg(all(target_os = "ios", feature = "ndef-write"))]
pub use ndef_write::{CoreNfcNdefWriteBackend, CoreNfcNdefWriteFuture};
