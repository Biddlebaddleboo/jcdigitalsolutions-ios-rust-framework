#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Bounded synchronous UIKit accessibility metadata plus point-in-time accessibility status queries for iOS apps."]

#[cfg(any(target_os = "ios", test))]
mod container_type;
#[cfg(any(target_os = "ios", test))]
mod expanded_status;
#[cfg(any(target_os = "ios", test))]
mod navigation_style;
#[cfg(any(target_os = "ios", test))]
mod text;
#[cfg(any(target_os = "ios", test))]
mod textual_context;
#[cfg(any(target_os = "ios", test))]
mod traits;

#[cfg(any(target_os = "ios", test))]
pub use container_type::AccessibilityContainerType;
#[cfg(any(target_os = "ios", test))]
pub use expanded_status::AccessibilityExpandedStatus;
#[cfg(any(target_os = "ios", test))]
pub use navigation_style::AccessibilityNavigationStyle;
#[cfg(any(target_os = "ios", test))]
pub use textual_context::AccessibilityTextualContext;
#[cfg(any(target_os = "ios", test))]
pub use traits::AccessibilityTrait;

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{
    AccessibilityApiUnavailable, AccessibilityMetadata, assistive_touch_is_enabled,
    classic_invert_is_enabled, guided_access_is_enabled,
    reduce_motion_is_enabled, reduce_transparency_is_enabled, speak_screen_is_enabled,
    speak_selection_is_enabled, switch_control_is_running, voice_over_is_running,
};

#[cfg(test)]
mod tests {
    use super::text::map_optional_text;
    use super::traits::{AccessibilityTrait, fold_traits};

    fn fixture_mask(trait_: AccessibilityTrait) -> u64 {
        match trait_ {
            AccessibilityTrait::Button => 1 << 0,
            AccessibilityTrait::Link => 1 << 1,
            AccessibilityTrait::Header => 1 << 2,
            AccessibilityTrait::Image => 1 << 3,
            AccessibilityTrait::Selected => 1 << 4,
            AccessibilityTrait::StaticText => 1 << 5,
            AccessibilityTrait::SearchField => 1 << 6,
            AccessibilityTrait::Adjustable => 1 << 7,
            AccessibilityTrait::NotEnabled => 1 << 8,
            AccessibilityTrait::KeyboardKey => 1 << 9,
            AccessibilityTrait::UpdatesFrequently => 1 << 10,
            AccessibilityTrait::PlaysSound => 1 << 11,
            AccessibilityTrait::CausesPageTurn => 1 << 12,
            AccessibilityTrait::StartsMediaSession => 1 << 13,
            AccessibilityTrait::AllowsDirectInteraction => 1 << 14,
            AccessibilityTrait::SummaryElement => 1 << 15,
            AccessibilityTrait::TabBar => 1 << 16,
        }
    }

    #[test]
    fn absent_text_remains_absent_and_empty_text_remains_present() {
        assert_eq!(map_optional_text(None, str::len), None);
        assert_eq!(map_optional_text(Some(""), str::len), Some(0));
        assert_eq!(map_optional_text(Some("Save"), str::len), Some(4));
    }

    #[test]
    fn trait_sets_combine_and_empty_set_uses_the_supplied_none_value() {
        let traits = [AccessibilityTrait::Button, AccessibilityTrait::Header];
        assert_eq!(fold_traits(&traits, 0, fixture_mask), (1 << 0) | (1 << 2));
        assert_eq!(fold_traits(&[], 99, fixture_mask), 99);
    }

    #[test]
    fn trait_mapping_is_order_independent_and_uses_only_passed_traits() {
        let first = [AccessibilityTrait::Link, AccessibilityTrait::Selected];
        let reversed = [AccessibilityTrait::Selected, AccessibilityTrait::Link];
        assert_eq!(
            fold_traits(&first, 0, fixture_mask),
            fold_traits(&reversed, 0, fixture_mask)
        );
        assert_eq!(
            fold_traits(&[AccessibilityTrait::Image], 0, fixture_mask),
            1 << 3
        );
    }
}
