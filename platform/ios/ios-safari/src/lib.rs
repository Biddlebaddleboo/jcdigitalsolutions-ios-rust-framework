#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded in-app SafariServices HTTPS controller wrapper for iOS."]

mod platform;

pub use platform::{IosSafariViewController, SafariControllerError};
