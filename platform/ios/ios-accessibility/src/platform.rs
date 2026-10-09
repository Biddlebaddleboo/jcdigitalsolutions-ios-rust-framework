use core::marker::PhantomData;
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2_foundation::NSString;
use objc2_ui_kit::{
    NSObjectUIAccessibility, UIAccessibilityTraitAdjustable,
    UIAccessibilityTraitAllowsDirectInteraction, UIAccessibilityTraitButton,
    UIAccessibilityTraitCausesPageTurn, UIAccessibilityTraitHeader, UIAccessibilityTraitImage,
    UIAccessibilityTraitKeyboardKey, UIAccessibilityTraitLink, UIAccessibilityTraitNone,
    UIAccessibilityTraitNotEnabled, UIAccessibilityTraitPlaysSound,
    UIAccessibilityTraitSearchField, UIAccessibilityTraitSelected,
    UIAccessibilityTraitStartsMediaSession, UIAccessibilityTraitStaticText,
    UIAccessibilityTraitSummaryElement, UIAccessibilityTraitUpdatesFrequently,
    UIAccessibilityTraits, UIView,
};

use crate::text::map_optional_text;
use crate::traits::{AccessibilityTrait, fold_traits};

/// Synchronous accessibility metadata access to a caller-owned borrowed UIKit view.
///
/// This adapter never creates, retains, lays out, or renders the view. It is neither `Send` nor
/// `Sync`; construct, call, and drop it on UIKit's main thread. The view remains caller-owned and
/// must outlive the adapter.
pub struct AccessibilityMetadata<'view> {
    view: &'view UIView,
    main_thread: MainThreadMarker,
    _not_sync: PhantomData<*mut ()>,
}

impl<'view> AccessibilityMetadata<'view> {
    /// Borrow a view for synchronous metadata updates using the existing typed main-thread proof.
    pub fn new(view: &'view UIView, main_thread: MainThread) -> Self {
        Self {
            view,
            main_thread: main_thread.into_objc2(),
            _not_sync: PhantomData,
        }
    }

    /// Replace whether UIKit exposes this view as an accessibility element.
    pub fn set_element(&self, is_element: bool) {
        self.view
            .setIsAccessibilityElement(is_element, self.main_thread);
    }

    /// Replace the label; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_label(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityLabel(value.as_deref(), self.main_thread);
    }

    /// Replace the hint; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_hint(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityHint(value.as_deref(), self.main_thread);
    }

    /// Replace the value; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_value(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityValue(value.as_deref(), self.main_thread);
    }

    /// Replace the complete trait mask; an empty slice clears all traits to UIKit's `None` value.
    ///
    /// This does not merge with the view's current traits or preserve standard-control defaults.
    /// Supply the complete desired set when calling this method.
    /// If the set includes `AccessibilityTrait::Adjustable`, the caller must ensure the view
    /// implements UIKit's `accessibilityIncrement` and `accessibilityDecrement` methods.
    /// If the set includes `AccessibilityTrait::NotEnabled`, the caller must ensure the view is
    /// already disabled or otherwise does not respond to user interaction; this setter changes
    /// metadata only.
    /// If the set includes `AccessibilityTrait::KeyboardKey`, the caller must provide the key
    /// behavior; this setter adds no key event handling.
    /// If the set includes `AccessibilityTrait::UpdatesFrequently`, the caller owns label/value
    /// updates and notification policy; this setter guarantees no polling behavior or timing.
    /// If the set includes `AccessibilityTrait::PlaysSound`, the caller must provide the sound on
    /// activation; this setter adds no activation or audio behavior.
    /// If the set includes `AccessibilityTrait::CausesPageTurn`, the caller must provide its own
    /// `accessibilityScroll:` handling and update the represented page; this setter implements no
    /// page behavior.
    /// If the set includes `AccessibilityTrait::StartsMediaSession`, the caller's activation must
    /// start the media session; this setter adds no media or audio behavior.
    /// If the set includes `AccessibilityTrait::AllowsDirectInteraction`, the caller must provide
    /// direct-touch behavior; this setter routes no touch events and configures no direct-touch
    /// options.
    /// If the set includes `AccessibilityTrait::SummaryElement`, the caller must provide a summary
    /// of current app conditions, settings, or state; this setter does not control when it is read.
    pub fn set_traits(&self, traits: &[AccessibilityTrait]) {
        // SAFETY: `UIAccessibilityTraitNone` is a public immutable UIKit constant from the active
        // SDK's `UIAccessibilityConstants.h` and is linked from UIKit.
        let mask = fold_traits(traits, unsafe { UIAccessibilityTraitNone }, native_trait);
        self.view.setAccessibilityTraits(mask, self.main_thread);
    }
}

fn native_trait(trait_: AccessibilityTrait) -> UIAccessibilityTraits {
    // SAFETY: Each value is a public immutable UIKit constant declared in the active SDK's
    // UIAccessibilityConstants.h and linked from UIKit. The set is limited to these declarations.
    unsafe {
        match trait_ {
            AccessibilityTrait::Button => UIAccessibilityTraitButton,
            AccessibilityTrait::Link => UIAccessibilityTraitLink,
            AccessibilityTrait::Header => UIAccessibilityTraitHeader,
            AccessibilityTrait::Image => UIAccessibilityTraitImage,
            AccessibilityTrait::Selected => UIAccessibilityTraitSelected,
            AccessibilityTrait::StaticText => UIAccessibilityTraitStaticText,
            AccessibilityTrait::SearchField => UIAccessibilityTraitSearchField,
            AccessibilityTrait::Adjustable => UIAccessibilityTraitAdjustable,
            AccessibilityTrait::NotEnabled => UIAccessibilityTraitNotEnabled,
            AccessibilityTrait::KeyboardKey => UIAccessibilityTraitKeyboardKey,
            AccessibilityTrait::UpdatesFrequently => UIAccessibilityTraitUpdatesFrequently,
            AccessibilityTrait::PlaysSound => UIAccessibilityTraitPlaysSound,
            AccessibilityTrait::CausesPageTurn => UIAccessibilityTraitCausesPageTurn,
            AccessibilityTrait::StartsMediaSession => UIAccessibilityTraitStartsMediaSession,
            AccessibilityTrait::AllowsDirectInteraction => {
                UIAccessibilityTraitAllowsDirectInteraction
            }
            AccessibilityTrait::SummaryElement => UIAccessibilityTraitSummaryElement,
        }
    }
}
