#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded native UIKit view and control backend for iOS."]

#[cfg(any(test, target_os = "ios"))]
mod disclosure_policy;
/// Bounded CoreGraphics geometry operations over portable frames.
#[cfg(target_os = "ios")]
pub mod geometry;
#[cfg(target_os = "ios")]
mod grid;
#[cfg(any(test, target_os = "ios"))]
mod grid_policy;
#[cfg(any(test, target_os = "ios"))]
mod keyboard_action_policy;
#[cfg(any(test, target_os = "ios"))]
mod layout_policy;
#[cfg(any(test, target_os = "ios"))]
mod list_policy;
#[cfg(target_os = "ios")]
mod platform;
#[cfg(any(test, target_os = "ios"))]
mod scroll_policy;
#[cfg(any(test, target_os = "ios"))]
mod stack_spacing;
#[cfg(any(test, target_os = "ios"))]
mod system_symbol_policy;
#[cfg(any(test, target_os = "ios"))]
mod text_conversion;
/// System UI font vertical metrics from CoreText.
#[cfg(target_os = "ios")]
pub mod text_metrics;
#[cfg(any(test, target_os = "ios"))]
mod text_policy;
#[cfg(any(test, target_os = "ios"))]
mod toggle_policy;

#[cfg(target_os = "ios")]
pub use grid::IosGridView;
#[cfg(target_os = "ios")]
pub use platform::IosActivityIndicator;
#[cfg(target_os = "ios")]
pub use platform::IosDisclosureGroup;
#[cfg(target_os = "ios")]
pub use platform::{
    IosButton, IosImageView, IosLabel, IosScrollView, IosStackAxis, IosStackView, IosTextField,
    IosToggle, IosUiBackend, IosView,
};
#[cfg(target_os = "ios")]
pub use platform::{IosListRow, IosListView};
