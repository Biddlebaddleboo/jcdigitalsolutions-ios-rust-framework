#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded external HTTPS URL-handler requests through public iOS UIKit APIs."]

mod platform;

pub use platform::{OpenError, open_external_https_uri};
