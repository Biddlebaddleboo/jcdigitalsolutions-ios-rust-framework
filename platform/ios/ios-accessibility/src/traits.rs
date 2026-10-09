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
    /// Mark a value that a user can adjust through a range.
    ///
    /// The caller must ensure the view implements UIKit's `accessibilityIncrement` and
    /// `accessibilityDecrement` methods. This adapter sets the trait only and provides no callbacks.
    Adjustable,
    /// Mark the accessibility element as not enabled.
    ///
    /// This adapter sets metadata only; the caller must use this only when the underlying view or
    /// control is already disabled or otherwise does not respond to user interaction.
    NotEnabled,
    /// Mark the accessibility element as a keyboard key.
    ///
    /// The caller must provide the key behavior; this adapter sets metadata only and adds no key
    /// event handling.
    KeyboardKey,
    /// Mark an element whose label or value updates frequently enough that assistive apps may poll
    /// for changes instead of handling continual change notifications.
    ///
    /// This adapter sets metadata only. The caller owns label/value updates and notification
    /// policy; this trait provides no guarantee that an assistive app polls or when.
    UpdatesFrequently,
    /// Mark an accessibility element that plays its own sound when activated.
    ///
    /// The caller must provide that activation sound; this adapter sets metadata only and adds no
    /// activation or audio behavior.
    PlaysSound,
    /// Mark an accessibility element that represents a page of content in a page sequence.
    ///
    /// The caller must provide next-page behavior through its own `accessibilityScroll:`
    /// implementation and update the represented page; this adapter sets metadata only.
    CausesPageTurn,
    /// Mark an element whose activation starts a media session that should not be interrupted.
    ///
    /// The caller must start the media session; this adapter sets metadata only and adds no media
    /// or audio behavior.
    StartsMediaSession,
    /// Mark an accessibility element that represents a directly touch-interactive object.
    ///
    /// The caller must provide the touch interaction; this adapter sets metadata only and routes no
    /// touch events.
    AllowsDirectInteraction,
    /// Mark an element that provides an app-start summary of current conditions, settings, or state.
    ///
    /// The caller must provide a real summary; this adapter sets metadata only and does not control
    /// when UIKit or assistive technology reads or presents it.
    SummaryElement,
    /// Mark a view that represents an ordered list of tabs.
    ///
    /// This iOS 10.0 trait is for a tab-bar container; Apple says the view must not be an
    /// accessibility element. The caller must set `isAccessibilityElement` to `false`; this
    /// adapter sets the trait only and does not create or order tab children.
    TabBar,
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
