#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded HTTPS `WKWebView` adapter with navigation controls and no native WebKit escape handle."]

mod platform;

pub use platform::IosWebView;
