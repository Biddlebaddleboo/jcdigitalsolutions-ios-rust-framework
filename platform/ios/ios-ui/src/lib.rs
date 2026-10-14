#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded native UIKit view and control backend for iOS."]

#[cfg(target_os = "ios")]
mod alert;
#[cfg(any(test, target_os = "ios"))]
mod alert_policy;
#[cfg(any(test, target_os = "ios"))]
mod alert_state;
#[cfg(any(test, target_os = "ios"))]
mod collection_grid_policy;
#[cfg(any(test, target_os = "ios"))]
mod context_menu_policy;
#[cfg(target_os = "ios")]
mod custom_render_accessibility;
#[cfg(any(test, target_os = "ios"))]
mod custom_render_accessibility_policy;
#[cfg(any(test, target_os = "ios"))]
mod custom_render_frame;
#[cfg(target_os = "ios")]
mod custom_render_host;
#[cfg(all(target_os = "ios", feature = "custom-render-metal"))]
mod custom_render_metal;
#[cfg(target_os = "ios")]
mod custom_render_session;
#[cfg(any(test, target_os = "ios"))]
mod custom_render_session_policy;
#[cfg(target_os = "ios")]
mod custom_render_text_input;
#[cfg(any(test, target_os = "ios"))]
mod custom_render_text_policy;
#[cfg(target_os = "ios")]
mod custom_render_touch;
#[cfg(any(test, target_os = "ios"))]
mod custom_render_touch_state;
#[cfg(any(test, target_os = "ios"))]
mod disclosure_policy;
#[cfg(target_os = "ios")]
mod display_link;
#[cfg(any(test, target_os = "ios"))]
mod display_link_policy;
#[cfg(any(test, target_os = "ios"))]
mod font_traits_policy;
/// Bounded CoreGraphics geometry operations over portable frames.
#[cfg(target_os = "ios")]
pub mod geometry;
#[cfg(target_os = "ios")]
mod grid;
#[cfg(any(test, target_os = "ios"))]
mod grid_policy;
#[cfg(any(test, target_os = "ios"))]
mod integer_text_policy;
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
mod slider_policy;
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
mod text_style_policy;
#[cfg(any(test, target_os = "ios"))]
mod toggle_policy;

#[cfg(target_os = "ios")]
pub use alert::{
    IosAlertActionStyle, IosAlertError, IosAlertKeyboardType, IosAlertResult, IosAlertSession,
    IosAlertStartError, IosAlertStyle, IosAlertTextFieldConfig,
};
#[cfg(target_os = "ios")]
pub use collection_grid_policy::{
    IosCollectionGridEvent, IosCollectionGridInsets, IosCollectionGridItem,
    IosCollectionGridItemContent, IosCollectionGridLayout, IosCollectionGridSection,
    IosCollectionGridSectionHeader,
};
#[cfg(target_os = "ios")]
pub use custom_render_accessibility::{
    IosCustomRenderAccessibility, SemanticAccessibilityAction, SemanticAccessibilityNode,
};
#[cfg(any(test, target_os = "ios"))]
pub use custom_render_accessibility_policy::{
    AccessibilityActionEvent, AccessibilityActionId, AccessibilityNodeId, SemanticActionKind,
    SemanticFrame, SemanticTreeError,
};
#[cfg(target_os = "ios")]
pub use custom_render_host::{
    CustomRenderHostError, FrameValidationError, HostApplicationState, HostGeometry,
    IosCustomRenderHost, Mono1Frame, Mono1FrameError, PixelDamageRect, Rgba8Frame, Rgba8FrameError,
};
#[cfg(all(target_os = "ios", feature = "custom-render-metal"))]
pub use custom_render_metal::{
    Bgra8Frame, Bgra8FrameError, CustomMetalHostError, IosCustomMetalHost,
};
#[cfg(target_os = "ios")]
pub use custom_render_session::{IosCustomRenderSession, IosCustomRenderSessionError};
#[cfg(target_os = "ios")]
pub use custom_render_text_input::{
    IosCustomTextInput, IosCustomTextInputSnapshot, IosKeyboardFrame, IosTextCapitalization,
    IosTextInputKeyboardType, IosTextInputTraits,
};
#[cfg(target_os = "ios")]
pub use custom_render_text_policy::{Utf16Range, Utf16RangeError};
#[cfg(target_os = "ios")]
pub use custom_render_touch::{
    IosCustomRenderTouchInput, IosTouchEvent, IosTouchInputFault, IosTouchKind, IosTouchPhase,
};
#[cfg(target_os = "ios")]
pub use display_link::{IosDisplayFrame, IosDisplayLink};
#[cfg(any(test, target_os = "ios"))]
pub use font_traits_policy::{IosFontDesign, IosFontWeight};
#[cfg(target_os = "ios")]
pub use grid::IosGridView;
#[cfg(target_os = "ios")]
pub use platform::IosActivityIndicator;
#[cfg(target_os = "ios")]
pub use platform::IosCollectionGrid;
#[cfg(target_os = "ios")]
pub use platform::IosDisclosureGroup;
#[cfg(target_os = "ios")]
pub use platform::{
    IosButton, IosImageView, IosLabel, IosScrollView, IosSelectableTextView, IosSlider,
    IosStackAlignment, IosStackAxis, IosStackDistribution, IosStackView, IosTextEditor,
    IosTextField, IosToggle, IosUiBackend, IosView,
};
#[cfg(target_os = "ios")]
pub use platform::{IosContextMenu, IosContextMenuAction};
#[cfg(target_os = "ios")]
pub use platform::{IosListRow, IosListSection, IosListView, IosSectionedListView};
#[cfg(target_os = "ios")]
pub use slider_policy::{IosSliderConfig, IosSliderRange, IosSliderReporting};
#[cfg(any(test, target_os = "ios"))]
pub use text_style_policy::{IosSemanticTextColor, IosSystemTextStyle};
