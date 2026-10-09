/// A bounded set of UIKit accessibility navigation styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityNavigationStyle {
    /// Let the assistive technology determine how the receiver's elements should be navigated.
    ///
    /// This is UIKit's default.
    Automatic,
    /// Navigate the receiver's elements as separate elements.
    Separate,
    /// Combine the receiver's elements and navigate them as one item.
    Combined,
}
