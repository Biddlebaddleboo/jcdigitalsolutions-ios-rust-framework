#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded native UIKit view and control backend for iOS."]

/// Bounded CoreGraphics geometry operations over portable frames.
#[cfg(target_os = "ios")]
pub mod geometry;
#[cfg(target_os = "ios")]
mod platform;
#[cfg(any(test, target_os = "ios"))]
mod text_conversion;
/// System UI font vertical metrics from CoreText.
#[cfg(target_os = "ios")]
pub mod text_metrics;

#[cfg(target_os = "ios")]
pub use platform::{IosButton, IosImageView, IosLabel, IosTextField, IosUiBackend, IosView};
