/// A bounded subset of UIKit accessibility container types supported by this adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityContainerType {
    /// Mark no additional data-based container type.
    Unspecified,
    /// Mark a container as a list.
    List,
    /// Mark a container as a landmark.
    Landmark,
    /// Mark a semantic group; callers must use this value only on iOS 13 or later.
    SemanticGroup,
}
