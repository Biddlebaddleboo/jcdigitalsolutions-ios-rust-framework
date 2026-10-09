#![deny(missing_docs)]
#![doc = "Strict, caller-owned Foundation URL values for portable RFC 3986 URIs."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{NativeUrl, NativeUrlError};
