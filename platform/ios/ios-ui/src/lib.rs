#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded native UIKit container, label, and button backend for iOS."]

/// Bounded CoreGraphics geometry operations over portable frames.
pub mod geometry;
mod platform;
/// System UI font vertical metrics from CoreText.
pub mod text_metrics;

pub use platform::{IosButton, IosLabel, IosUiBackend, IosView};
