/// A bounded mapping of UIKit expanded-state accessibility metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityExpandedStatus {
    /// The element has no supported expanded state.
    Unsupported,
    /// The element's content is expanded.
    Expanded,
    /// The element's content is collapsed.
    Collapsed,
}
