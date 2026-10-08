/// A bounded set of standard UIKit accessibility traits supported by this adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityTrait {
    /// Treat the view as a button.
    Button,
    /// Treat the view as a link.
    Link,
    /// Treat the view as a section header.
    Header,
    /// Treat the view as an image.
    Image,
    /// Mark the view as selected.
    Selected,
    /// Treat the view as static text.
    StaticText,
    /// Treat the view as a search field.
    SearchField,
}

pub(crate) fn fold_traits(
    traits: &[AccessibilityTrait],
    none: u64,
    mut map: impl FnMut(AccessibilityTrait) -> u64,
) -> u64 {
    traits
        .iter()
        .copied()
        .fold(none, |mask, trait_| mask | map(trait_))
}
