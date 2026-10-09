use core::marker::PhantomData;
use ios_runtime::main_thread::MainThread;
use objc2::runtime::NSObjectProtocol;
use objc2::{MainThreadMarker, rc::Retained, sel};
use objc2_core_foundation::{CGPoint, CGRect};
use objc2_foundation::{NSArray, NSAttributedString, NSCopying, NSString};
use objc2_ui_kit::{
    NSObjectUIAccessibility, NSObjectUIAccessibilityContainer, NSObjectUIAccessibilityFocus,
    UIAccessibilityButtonShapesEnabled,
    UIAccessibilityContainerType as NativeAccessibilityContainerType,
    UIAccessibilityContentSizeCategoryImageAdjusting,
    UIAccessibilityConvertFrameToScreenCoordinates, UIAccessibilityConvertPathToScreenCoordinates,
    UIAccessibilityDarkerSystemColorsEnabled, UIAccessibilityDirectTouchOptions,
    UIAccessibilityExpandedStatus as NativeAccessibilityExpandedStatus,
    UIAccessibilityHearingDeviceEar as NativeHearingDeviceEar,
    UIAccessibilityHearingDevicePairedEar, UIAccessibilityIdentification,
    UIAccessibilityIsAssistiveTouchRunning, UIAccessibilityIsBoldTextEnabled,
    UIAccessibilityIsClosedCaptioningEnabled, UIAccessibilityIsGrayscaleEnabled,
    UIAccessibilityIsGuidedAccessEnabled, UIAccessibilityIsInvertColorsEnabled,
    UIAccessibilityIsMonoAudioEnabled, UIAccessibilityIsOnOffSwitchLabelsEnabled,
    UIAccessibilityIsReduceMotionEnabled, UIAccessibilityIsReduceTransparencyEnabled,
    UIAccessibilityIsShakeToUndoEnabled, UIAccessibilityIsSpeakScreenEnabled,
    UIAccessibilityIsSpeakSelectionEnabled, UIAccessibilityIsSwitchControlRunning,
    UIAccessibilityIsVideoAutoplayEnabled, UIAccessibilityIsVoiceOverRunning,
    UIAccessibilityNavigationStyle as NativeAccessibilityNavigationStyle,
    UIAccessibilityPrefersCrossFadeTransitions, UIAccessibilityShouldDifferentiateWithoutColor,
    UIAccessibilityTextualContext as NativeAccessibilityTextualContext,
    UIAccessibilityTextualContextConsole, UIAccessibilityTextualContextFileSystem,
    UIAccessibilityTextualContextMessaging, UIAccessibilityTextualContextNarrative,
    UIAccessibilityTextualContextSourceCode, UIAccessibilityTextualContextSpreadsheet,
    UIAccessibilityTextualContextWordProcessing, UIAccessibilityTraitAdjustable,
    UIAccessibilityTraitAllowsDirectInteraction, UIAccessibilityTraitButton,
    UIAccessibilityTraitCausesPageTurn, UIAccessibilityTraitHeader, UIAccessibilityTraitImage,
    UIAccessibilityTraitKeyboardKey, UIAccessibilityTraitLink, UIAccessibilityTraitNone,
    UIAccessibilityTraitNotEnabled, UIAccessibilityTraitPlaysSound,
    UIAccessibilityTraitSearchField, UIAccessibilityTraitSelected,
    UIAccessibilityTraitStartsMediaSession, UIAccessibilityTraitStaticText,
    UIAccessibilityTraitSummaryElement, UIAccessibilityTraitTabBar,
    UIAccessibilityTraitUpdatesFrequently, UIAccessibilityTraits, UIBezierPath, UIButton,
    UIContentSizeCategory, UIGuidedAccessRestrictionState as NativeGuidedAccessRestrictionState,
    UIImageView, UITraitEnvironment, UIUserInterfaceLayoutDirection as NativeLayoutDirection,
    UIUserInterfaceStyle as NativeInterfaceStyle, UIView,
};

use crate::container_type::AccessibilityContainerType;
use crate::expanded_status::AccessibilityExpandedStatus;
use crate::navigation_style::AccessibilityNavigationStyle;
use crate::text::map_optional_text;
use crate::textual_context::AccessibilityTextualContext;
use crate::traits::{AccessibilityTrait, fold_traits};

/// A UIKit accessibility API or optional property selector is unavailable on this iOS runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessibilityApiUnavailable;

/// The current ear-side pairing status that UIKit reports for Made for iPhone hearing aids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HearingDevicePairingStatus {
    /// UIKit reports neither ear as paired.
    None,
    /// UIKit reports the left ear as paired.
    Left,
    /// UIKit reports the right ear as paired.
    Right,
    /// UIKit reports both ears as paired.
    Both,
    /// UIKit returned bits not named by the SDK constants known to this crate.
    Unknown(u64),
}

/// A Guided Access restriction state returned by UIKit for a caller-supplied identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GuidedAccessRestrictionState {
    /// UIKit reports that the restriction allows the associated operation.
    Allow,
    /// UIKit reports that the restriction denies the associated operation.
    Deny,
    /// UIKit returned a state value not named by the SDK constants known to this crate.
    Unknown(isize),
}

/// UIKit's current effective direction for arranging one view's immediate content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessibilityLayoutDirection {
    /// UIKit reports left-to-right layout direction.
    LeftToRight,
    /// UIKit reports right-to-left layout direction.
    RightToLeft,
    /// UIKit returned a direction value not named by the SDK constants known to this crate.
    Unknown(isize),
}

/// UIKit's current resolved interface style for one view's trait collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResolvedUserInterfaceStyle {
    /// UIKit reports no specific interface style.
    Unspecified,
    /// UIKit reports a light interface style.
    Light,
    /// UIKit reports a dark interface style.
    Dark,
    /// UIKit returned a style value not named by the SDK constants known to this crate.
    Unknown(isize),
}

/// The override value UIKit stores on one view, distinct from its resolved trait style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UserInterfaceStyleOverride {
    /// UIKit reports no local override; the view inherits from a parent view or view controller.
    Unspecified,
    /// UIKit reports a local light-style override.
    Light,
    /// UIKit reports a local dark-style override.
    Dark,
    /// UIKit returned a style value not named by the SDK constants known to this crate.
    Unknown(isize),
}

/// Read the current VoiceOver enabled/running state on UIKit's main actor.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`. This is one synchronous
/// snapshot; it does not observe later status changes, request a permission, or perform a UI action.
pub fn voice_over_is_running(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsVoiceOverRunning()
}

/// Read the current Switch Control enabled state on UIKit's main actor.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`. This is one synchronous
/// snapshot and may only be called on iOS 8.0 or later. It does not observe later status changes,
/// request a permission, or perform a UI action.
pub fn switch_control_is_running(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsSwitchControlRunning()
}

/// Read whether the app is currently running under Guided Access.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 6.0 or
/// later. This is one synchronous snapshot; it does not start or end a Guided Access session or
/// observe later status changes.
pub fn guided_access_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsGuidedAccessEnabled()
}

/// Read the current Guided Access state for one of the app's registered restriction identifiers.
///
/// Pass an identifier that the host app lists through its
/// `UIGuidedAccessRestrictionDelegate`. Call only on iOS 7.0 or later and pass a proof from
/// `ios_runtime::main_thread::MainThread::current`. `Allow` does not prove that the identifier is
/// registered or Guided Access is active. `Deny` is a state snapshot only; this crate does not
/// enforce it, and the host app must remove or disable the restricted operation from all UI paths.
/// Unknown native values are preserved in `GuidedAccessRestrictionState::Unknown`.
pub fn guided_access_restriction_state(
    restriction_identifier: &NSString,
    _main_thread: &MainThread,
) -> Result<GuidedAccessRestrictionState, AccessibilityApiUnavailable> {
    if !objc2::available!(ios = 7.0, ..) {
        return Err(AccessibilityApiUnavailable);
    }
    let native = NativeGuidedAccessRestrictionState::for_identifier(restriction_identifier);
    Ok(match native {
        NativeGuidedAccessRestrictionState::Allow => GuidedAccessRestrictionState::Allow,
        NativeGuidedAccessRestrictionState::Deny => GuidedAccessRestrictionState::Deny,
        unknown => GuidedAccessRestrictionState::Unknown(unknown.0),
    })
}

/// Read whether UIKit reports the Increase Contrast setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes,
/// mutate colors, or claim that UIKit applies contrast to app-owned custom drawing.
pub fn increase_contrast_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityDarkerSystemColorsEnabled()
}

/// Read UIKit's reported Color Filters or Grayscale preference state.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous setting snapshot; it does not identify the active filter, report
/// rendered colors, observe later status changes, or alter display output.
pub fn color_filters_or_grayscale_preference_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsGrayscaleEnabled()
}

/// Read whether UIKit reports Auto-Play Video Previews as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 13.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// report or control playback behavior for app video content.
pub fn video_autoplay_previews_are_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsVideoAutoplayEnabled()
}

/// Read UIKit's current Made for iPhone hearing-aid ear-side pairing status.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 10.0 or
/// later. This is one synchronous status snapshot; it does not expose device identity, report
/// connection, streaming, or audio-route state, prompt the user, observe notifications, or control
/// audio. Unknown native bit patterns are preserved in `HearingDevicePairingStatus::Unknown`.
pub fn hearing_device_paired_ear(_main_thread: &MainThread) -> HearingDevicePairingStatus {
    let native = UIAccessibilityHearingDevicePairedEar();
    let bits = native.bits();
    if bits == NativeHearingDeviceEar::None.bits() {
        HearingDevicePairingStatus::None
    } else if bits == NativeHearingDeviceEar::Left.bits() {
        HearingDevicePairingStatus::Left
    } else if bits == NativeHearingDeviceEar::Right.bits() {
        HearingDevicePairingStatus::Right
    } else if bits == NativeHearingDeviceEar::Both.bits() {
        HearingDevicePairingStatus::Both
    } else {
        HearingDevicePairingStatus::Unknown(bits as u64)
    }
}

/// Read whether AssistiveTouch is enabled when UIKit can report it accurately.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 10.0 or
/// later. Apple documents that the AssistiveTouch query always returns false outside Guided Access;
/// `None` means the app is not in Guided Access, while `Some(bool)` preserves the query result.
/// This snapshot does not observe changes, request permission, or control either feature.
pub fn assistive_touch_is_enabled(_main_thread: &MainThread) -> Option<bool> {
    if !guided_access_is_enabled(_main_thread) {
        return None;
    }
    Some(UIAccessibilityIsAssistiveTouchRunning())
}

/// Read whether UIKit reports the system Reduce Motion preference as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous snapshot; it does not observe later status changes or alter
/// animation behavior.
pub fn reduce_motion_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsReduceMotionEnabled()
}

/// Read whether UIKit reports the system Reduce Transparency preference as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous snapshot; it does not observe later status changes or alter
/// visual effects.
pub fn reduce_transparency_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsReduceTransparencyEnabled()
}

/// Read whether UIKit reports the system Speak Screen setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous setting snapshot; it does not invoke Speak Screen, observe later
/// status changes, or perform a UI action.
pub fn speak_screen_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsSpeakScreenEnabled()
}

/// Read whether UIKit reports the system Speak Selection setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous setting snapshot; it does not invoke Speak Selection, observe
/// later status changes, or perform a UI action.
pub fn speak_selection_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsSpeakSelectionEnabled()
}

/// Read whether UIKit reports the Classic Invert setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 6.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// alter display colors.
pub fn classic_invert_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsInvertColorsEnabled()
}

/// Read whether UIKit reports the system On/Off Labels setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 13.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// alter switch labels.
pub fn on_off_switch_labels_are_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsOnOffSwitchLabelsEnabled()
}

/// Read whether UIKit reports the system Bold Text setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 8.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// modify text rendering.
pub fn bold_text_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsBoldTextEnabled()
}

/// Read whether UIKit reports the system Button Shapes setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 14.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// promise rendered-button appearance. Apple deprecated this UIKit API in iOS 26.1 in favor of
/// `AXShowBordersEnabled`.
pub fn button_shapes_are_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityButtonShapesEnabled()
}

/// Read whether UIKit reports the system Closed Captions + SDH setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 5.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// render captions.
pub fn closed_captioning_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsClosedCaptioningEnabled()
}

/// Read whether UIKit reports the system Mono Audio setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 5.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// change audio output.
pub fn mono_audio_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsMonoAudioEnabled()
}

/// Read whether UIKit reports the system Shake to Undo setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 9.0 or
/// later. This is one synchronous setting snapshot; it does not trigger an undo action, observe
/// later status changes, or perform a UI action.
pub fn shake_to_undo_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityIsShakeToUndoEnabled()
}

/// Read whether UIKit reports the Differentiate Without Color setting as enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 13.0 or
/// later. This is one synchronous setting snapshot; it does not observe later status changes or
/// change the app's visual presentation.
pub fn differentiate_without_color_is_enabled(_main_thread: &MainThread) -> bool {
    UIAccessibilityShouldDifferentiateWithoutColor()
}

/// Read whether UIKit reports Reduce Motion with Prefer Cross-Fade Transitions enabled.
///
/// Pass a proof from `ios_runtime::main_thread::MainThread::current`, and call only on iOS 14.0 or
/// later. Apple defines this as a combined setting snapshot; it does not change app animations or
/// observe later status changes.
pub fn cross_fade_transitions_are_preferred(_main_thread: &MainThread) -> bool {
    UIAccessibilityPrefersCrossFadeTransitions()
}

enum AccessibilityImageSizingTarget<'view> {
    ImageView(&'view UIImageView),
    Button(&'view UIButton),
}

/// Synchronous image-size adjustment access for a borrowed `UIImageView` or `UIButton`.
///
/// This adapter never creates or retains the target. It is neither `Send` nor `Sync`; construct,
/// call, and drop it on UIKit's main thread. The target remains caller-owned and must outlive the
/// adapter.
pub struct AccessibilityImageSizing<'view> {
    target: AccessibilityImageSizingTarget<'view>,
    _main_thread: MainThreadMarker,
    _not_sync: PhantomData<*mut ()>,
}

impl<'view> AccessibilityImageSizing<'view> {
    /// Borrow a caller-owned image view for iOS 11.0+ image-size adjustment access.
    ///
    /// The caller must provide the main-thread proof and must call the adapter only on iOS 11.0 or
    /// later. UIKit's image scaling requires host-selected scalable content and a scaling content
    /// mode; behavior is undefined for a `UIImageView` whose `contentMode` does not scale its image.
    pub fn for_image_view(view: &'view UIImageView, main_thread: MainThread) -> Self {
        Self {
            target: AccessibilityImageSizingTarget::ImageView(view),
            _main_thread: main_thread.into_objc2(),
            _not_sync: PhantomData,
        }
    }

    /// Borrow a caller-owned button for iOS 11.0+ image-size adjustment access.
    ///
    /// The caller must provide the main-thread proof and must call the adapter only on iOS 11.0 or
    /// later. UIKit scales the button's image, not its background image; the caller owns the image,
    /// scalable asset choice, and layout.
    pub fn for_button(view: &'view UIButton, main_thread: MainThread) -> Self {
        Self {
            target: AccessibilityImageSizingTarget::Button(view),
            _main_thread: main_thread.into_objc2(),
            _not_sync: PhantomData,
        }
    }

    /// Read UIKit's current image-size adjustment property.
    ///
    /// Call only on iOS 11.0 or later. This reads the property; it does not establish that the
    /// supplied image can scale well or guarantee a resulting layout or assistive-technology
    /// output.
    pub fn adjusts_image_size_for_accessibility_content_size_category(&self) -> bool {
        match &self.target {
            AccessibilityImageSizingTarget::ImageView(view) => {
                view.adjustsImageSizeForAccessibilityContentSizeCategory()
            }
            AccessibilityImageSizingTarget::Button(view) => {
                view.adjustsImageSizeForAccessibilityContentSizeCategory()
            }
        }
    }

    /// Replace UIKit's image-size adjustment property.
    ///
    /// Call only on iOS 11.0 or later. Enabling it changes image sizing for accessibility content
    /// size categories. The host must choose scalable content and a suitable image content mode;
    /// this setter does not guarantee a resulting layout or assistive-technology output.
    pub fn set_adjusts_image_size_for_accessibility_content_size_category(&self, adjusts: bool) {
        match &self.target {
            AccessibilityImageSizingTarget::ImageView(view) => {
                view.setAdjustsImageSizeForAccessibilityContentSizeCategory(adjusts)
            }
            AccessibilityImageSizingTarget::Button(view) => {
                view.setAdjustsImageSizeForAccessibilityContentSizeCategory(adjusts)
            }
        }
    }
}

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

    /// Return UIKit's current `isAccessibilityElement` property value.
    ///
    /// This reports the property only; it does not report whether assistive technology currently
    /// exposes or can reach the view.
    pub fn is_element(&self) -> bool {
        self.view.isAccessibilityElement(self.main_thread)
    }

    /// Read whether UIKit reports an assistive technology focused on this view at this instant.
    ///
    /// The iOS 4.0 method is selector-checked. This is a point-in-time status query; it does not
    /// identify the assistive technology, observe focus changes, or move accessibility focus.
    pub fn accessibility_element_is_focused(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityElementIsFocused))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityElementIsFocused(self.main_thread))
    }

    /// Return owned identifiers for assistive technologies that UIKit reports as focused here.
    ///
    /// The iOS 9.0 method is selector-checked. `None` preserves a native `nil` result; strings in
    /// a returned set are copied to Rust-owned values and sorted lexicographically because the
    /// native set has no ordering. The result is a point-in-time snapshot, not a focus callback.
    pub fn accessibility_assistive_technology_focused_identifiers(
        &self,
    ) -> Result<Option<Vec<String>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAssistiveTechnologyFocusedIdentifiers))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let Some(native_identifiers) = self
            .view
            .accessibilityAssistiveTechnologyFocusedIdentifiers(self.main_thread)
        else {
            return Ok(None);
        };
        let native_identifiers = native_identifiers.allObjects();
        let mut identifiers = Vec::with_capacity(native_identifiers.count());
        for index in 0..native_identifiers.count() {
            identifiers.push(native_identifiers.objectAtIndex(index).to_string());
        }
        identifiers.sort_unstable();
        Ok(Some(identifiers))
    }

    /// Replace whether UIKit exposes this view as an accessibility element.
    pub fn set_element(&self, is_element: bool) {
        self.view
            .setIsAccessibilityElement(is_element, self.main_thread);
    }

    /// Read whether UIKit exempts this view from accessibility-requested color inversion.
    ///
    /// The iOS 11.0 property is selector-checked. This returns the property only; it does not
    /// report whether a color-inversion setting is enabled or which colors UIKit displays.
    pub fn accessibility_ignores_invert_colors(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityIgnoresInvertColors))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityIgnoresInvertColors())
    }

    /// Replace whether UIKit exempts this view subtree from accessibility-requested color inversion.
    ///
    /// Use `true` only when inversion damages this view's content. UIKit applies the property to
    /// this view and all subviews. The iOS 11.0 setter is selector-checked; this changes UIKit
    /// behavior but does not alter the user's accessibility setting.
    pub fn set_accessibility_ignores_invert_colors(
        &self,
        ignores_invert_colors: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityIgnoresInvertColors:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityIgnoresInvertColors(ignores_invert_colors);
        Ok(())
    }

    /// Read whether UIKit marks this view to show in the Large Content Viewer.
    ///
    /// The iOS 13.0 property is selector-checked. This reads the property only; it does not report
    /// whether the device viewer is enabled or an interaction is attached.
    pub fn shows_large_content_viewer(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(showsLargeContentViewer)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.showsLargeContentViewer())
    }

    /// Replace whether UIKit marks this view to show in the Large Content Viewer.
    ///
    /// The iOS 13.0 property defaults to `false` and is selector-checked. For this value to take
    /// effect, this view or an ancestor must have a host-owned `UILargeContentViewerInteraction`.
    /// This method does not add an interaction, present the viewer, or guarantee display.
    pub fn set_shows_large_content_viewer(
        &self,
        shows_large_content_viewer: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setShowsLargeContentViewer:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setShowsLargeContentViewer(shows_large_content_viewer);
        Ok(())
    }

    /// Read UIKit's current nullable Large Content Viewer title as owned Rust text.
    ///
    /// Call only on iOS 13.0 or later. The selector is checked so the package keeps its iOS 4.0
    /// floor; `None` preserves native `nil`, while `Some("")` preserves an empty title. UIKit may
    /// supply a default value for standard controls, so this does not prove caller assignment. It
    /// does not attach an interaction, present the viewer, or promise that UI appears.
    pub fn large_content_title(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(largeContentTitle)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.largeContentTitle().map(|title| title.to_string()))
    }

    /// Read UIKit's current effective layout direction for this view's immediate content.
    ///
    /// Call only on iOS 10.0 or later. This selector-guarded snapshot preserves unknown native
    /// values. UIKit warns that the direction does not necessarily propagate through the view's
    /// subtree; query each view whose immediate content the host arranges. This method does not
    /// perform layout, mutate the view, infer the app's general language direction, or observe
    /// later changes.
    pub fn effective_user_interface_layout_direction(
        &self,
    ) -> Result<AccessibilityLayoutDirection, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(effectiveUserInterfaceLayoutDirection))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let native = self.view.effectiveUserInterfaceLayoutDirection();
        Ok(if native == NativeLayoutDirection::LeftToRight {
            AccessibilityLayoutDirection::LeftToRight
        } else if native == NativeLayoutDirection::RightToLeft {
            AccessibilityLayoutDirection::RightToLeft
        } else {
            AccessibilityLayoutDirection::Unknown(native.0)
        })
    }

    /// Read UIKit's current resolved interface style from this view's trait collection.
    ///
    /// Call only on iOS 12.0 or later. The view's `traitCollection` and its
    /// `userInterfaceStyle` selector are checked so the package keeps its iOS 4.0 floor. The
    /// unsafe generated getter is called only while this adapter's `MainThread` proof remains
    /// valid. This is a point-in-time trait value, not the view's `overrideUserInterfaceStyle`,
    /// a guarantee about rendered colors, or an observer of later trait changes.
    pub fn resolved_user_interface_style(
        &self,
    ) -> Result<ResolvedUserInterfaceStyle, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(traitCollection)) {
            return Err(AccessibilityApiUnavailable);
        }
        let traits = self.view.traitCollection();
        if !traits.respondsToSelector(sel!(userInterfaceStyle)) {
            return Err(AccessibilityApiUnavailable);
        }
        // SAFETY: `AccessibilityMetadata` retains the constructor's main-thread proof and is
        // neither Send nor Sync; this synchronous getter therefore runs on the UIKit main thread.
        let native = unsafe { traits.userInterfaceStyle() };
        Ok(if native == NativeInterfaceStyle::Unspecified {
            ResolvedUserInterfaceStyle::Unspecified
        } else if native == NativeInterfaceStyle::Light {
            ResolvedUserInterfaceStyle::Light
        } else if native == NativeInterfaceStyle::Dark {
            ResolvedUserInterfaceStyle::Dark
        } else {
            ResolvedUserInterfaceStyle::Unknown(native.0)
        })
    }

    /// Read this view's own `overrideUserInterfaceStyle` property.
    ///
    /// Call only on iOS 13.0 or later. The selector is checked so the package keeps its iOS 4.0
    /// floor. `Unspecified` means UIKit inherits style from a parent view or view controller; it
    /// does not report the effective style, which is available separately from
    /// `resolved_user_interface_style()`. This getter does not mutate the view or promise rendered
    /// colors; this crate provides no setter for the override.
    pub fn override_user_interface_style(
        &self,
    ) -> Result<UserInterfaceStyleOverride, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(overrideUserInterfaceStyle))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let native = self.view.overrideUserInterfaceStyle();
        Ok(if native == NativeInterfaceStyle::Unspecified {
            UserInterfaceStyleOverride::Unspecified
        } else if native == NativeInterfaceStyle::Light {
            UserInterfaceStyleOverride::Light
        } else if native == NativeInterfaceStyle::Dark {
            UserInterfaceStyleOverride::Dark
        } else {
            UserInterfaceStyleOverride::Unknown(native.0)
        })
    }

    /// Read this view's current preferred content-size category as an owned native string.
    ///
    /// Call only on iOS 10.0 or later. The view's `traitCollection` and its
    /// `preferredContentSizeCategory` selector are checked so the package keeps its iOS 4.0 floor.
    /// This is the exact retained `UIContentSizeCategory` value in this view's trait environment;
    /// it is not necessarily a device-global setting. The getter does not scale fonts, observe
    /// Dynamic Type changes, or promise rendered text sizes.
    pub fn preferred_content_size_category(
        &self,
    ) -> Result<Retained<UIContentSizeCategory>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(traitCollection)) {
            return Err(AccessibilityApiUnavailable);
        }
        let traits = self.view.traitCollection();
        if !traits.respondsToSelector(sel!(preferredContentSizeCategory)) {
            return Err(AccessibilityApiUnavailable);
        }
        // SAFETY: `AccessibilityMetadata` retains the constructor's main-thread proof and is
        // neither Send nor Sync; this synchronous getter therefore runs on the UIKit main thread.
        Ok(unsafe { traits.preferredContentSizeCategory() })
    }

    /// Read whether UIKit marks accessible descendants of this view as hidden.
    ///
    /// The iOS 5.0 property is checked with `respondsToSelector:` first. This reads the property
    /// only; it does not report a VoiceOver or assistive-application result.
    pub fn accessibility_elements_hidden(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityElementsHidden))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityElementsHidden(self.main_thread))
    }

    /// Replace whether UIKit marks accessible descendants of this view as hidden.
    ///
    /// This does not set the view's own `isAccessibilityElement`, its visual `hidden` state, or
    /// interaction state. The iOS 5.0 property is checked with `respondsToSelector:` first.
    pub fn set_accessibility_elements_hidden(
        &self,
        hidden: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityElementsHidden:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityElementsHidden(hidden, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current `shouldGroupAccessibilityChildren` property.
    ///
    /// The iOS 6.0 property is checked with `respondsToSelector:` first. This reports the property
    /// only; it does not guarantee an assistive application's traversal order.
    pub fn should_group_accessibility_children(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(shouldGroupAccessibilityChildren))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.shouldGroupAccessibilityChildren(self.main_thread))
    }

    /// Replace UIKit's current `shouldGroupAccessibilityChildren` property.
    ///
    /// Use this only on a parent view whose children are a logical group. The iOS 6.0 property is
    /// checked with `respondsToSelector:` first; this does not guarantee an assistive application's
    /// grouping or traversal result.
    pub fn set_should_group_accessibility_children(
        &self,
        should_group: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setShouldGroupAccessibilityChildren:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setShouldGroupAccessibilityChildren(should_group, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current `accessibilityViewIsModal` property.
    ///
    /// The iOS 5.0 property is checked with `respondsToSelector:` first. This reports the property
    /// only; it does not report a current assistive-application navigation result.
    pub fn accessibility_view_is_modal(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityViewIsModal)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityViewIsModal(self.main_thread))
    }

    /// Replace UIKit's current `accessibilityViewIsModal` property.
    ///
    /// Use this only for a view that represents currently modal accessible content. The iOS 5.0
    /// property is checked with `respondsToSelector:` first; this does not present, dismiss, or
    /// otherwise make the view modal.
    pub fn set_accessibility_view_is_modal(
        &self,
        is_modal: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityViewIsModal:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityViewIsModal(is_modal, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current `accessibilityRespondsToUserInteraction` property.
    ///
    /// The iOS 13.0 property is checked with `respondsToSelector:` first. UIKit may derive its
    /// default from other accessibility properties; this reports the getter value only.
    pub fn accessibility_responds_to_user_interaction(
        &self,
    ) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityRespondsToUserInteraction))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityRespondsToUserInteraction(self.main_thread))
    }

    /// Replace UIKit's current `accessibilityRespondsToUserInteraction` property.
    ///
    /// Set `true` only if the accessible element actually performs an action in response to user
    /// interaction. The iOS 13.0 property is checked with `respondsToSelector:` first; this method
    /// adds no action handler or interaction behavior.
    pub fn set_accessibility_responds_to_user_interaction(
        &self,
        responds_to_interaction: bool,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityRespondsToUserInteraction:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityRespondsToUserInteraction(responds_to_interaction, self.main_thread);
        Ok(())
    }

    /// Replace the label; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_label(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityLabel(value.as_deref(), self.main_thread);
    }

    /// Read UIKit's current nullable `accessibilityLabel` property as an owned Rust string.
    ///
    /// The getter is selector-checked and preserves `nil` versus an empty string. It does not
    /// invoke the iOS 17 `accessibilityLabelBlock` or report assistive-application output.
    pub fn accessibility_label(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityLabel)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityLabel(self.main_thread)
            .map(|label| label.to_string()))
    }

    /// Return UIKit's current accessibility identifier as an owned Rust string.
    ///
    /// This iOS 5.0 property is selector-checked. The value is identifier metadata for uses such
    /// as UI Automation, not an accessibility label; it does not invoke the iOS 17
    /// `accessibilityIdentifierBlock` or report assistive output.
    pub fn accessibility_identifier(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityIdentifier)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityIdentifier()
            .map(|identifier| identifier.to_string()))
    }

    /// Replace UIKit's accessibility identifier; `None` clears it.
    ///
    /// This iOS 5.0 property is selector-checked and copied by UIKit. Identifiers are metadata for
    /// uses such as UI Automation; this setter does not change `accessibilityLabel`, invoke the
    /// iOS 17 `accessibilityIdentifierBlock`, validate uniqueness, or promise assistive output.
    pub fn set_accessibility_identifier(
        &self,
        identifier: Option<&str>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityIdentifier:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let identifier = map_optional_text(identifier, NSString::from_str);
        self.view.setAccessibilityIdentifier(identifier.as_deref());
        Ok(())
    }

    /// Replace UIKit's attributed accessibility label; `None` clears it.
    ///
    /// The iOS 11.0 property is selector-checked. UIKit copies the supplied attributed string and
    /// documents that setting this property also changes `accessibilityLabel`; this method makes
    /// no claim about speech attributes or assistive-application output.
    pub fn set_accessibility_attributed_label(
        &self,
        label: Option<&NSAttributedString>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityAttributedLabel:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityAttributedLabel(label, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current attributed accessibility label as an owned immutable value.
    ///
    /// The iOS 11.0 getter is selector-checked. This returns UIKit's attributed property only; it
    /// does not invoke the iOS 17 `accessibilityAttributedLabelBlock` or claim speech output.
    pub fn accessibility_attributed_label(
        &self,
    ) -> Result<Option<Retained<NSAttributedString>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedLabel))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityAttributedLabel(self.main_thread))
    }

    /// Replace the hint; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_hint(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityHint(value.as_deref(), self.main_thread);
    }

    /// Read UIKit's current nullable `accessibilityHint` property as an owned Rust string.
    ///
    /// The getter is selector-checked and preserves `nil` versus an empty string. It does not
    /// invoke the iOS 17 `accessibilityHintBlock` or report assistive-application output.
    pub fn accessibility_hint(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityHint)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityHint(self.main_thread)
            .map(|hint| hint.to_string()))
    }

    /// Replace UIKit's attributed accessibility hint; `None` clears it.
    ///
    /// The iOS 11.0 property is selector-checked. UIKit copies the supplied attributed string and
    /// documents that setting this property also changes `accessibilityHint`; this method makes
    /// no claim about speech attributes or assistive-application output.
    pub fn set_accessibility_attributed_hint(
        &self,
        hint: Option<&NSAttributedString>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityAttributedHint:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityAttributedHint(hint, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current attributed accessibility hint as an owned immutable value.
    ///
    /// The iOS 11.0 getter is selector-checked. This returns UIKit's attributed property only; it
    /// does not invoke the iOS 17 `accessibilityAttributedHintBlock` or claim speech output.
    pub fn accessibility_attributed_hint(
        &self,
    ) -> Result<Option<Retained<NSAttributedString>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedHint))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityAttributedHint(self.main_thread))
    }

    /// Replace the value; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_value(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityValue(value.as_deref(), self.main_thread);
    }

    /// Read UIKit's current nullable `accessibilityValue` property as an owned Rust string.
    ///
    /// The getter is selector-checked and preserves `nil` versus an empty string. It does not
    /// invoke the iOS 17 `accessibilityValueBlock` or report assistive-application output.
    pub fn accessibility_value(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityValue)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityValue(self.main_thread)
            .map(|value| value.to_string()))
    }

    /// Replace UIKit's attributed accessibility value; `None` clears it.
    ///
    /// The iOS 11.0 property is selector-checked. UIKit copies the supplied attributed string and
    /// documents that setting this property also changes `accessibilityValue`; this method makes
    /// no claim about speech attributes or assistive-application output.
    pub fn set_accessibility_attributed_value(
        &self,
        value: Option<&NSAttributedString>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityAttributedValue:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityAttributedValue(value, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current attributed accessibility value as an owned immutable value.
    ///
    /// The iOS 11.0 getter is selector-checked. This returns UIKit's attributed property only; it
    /// does not invoke the iOS 17 `accessibilityAttributedValueBlock` or claim speech output.
    pub fn accessibility_attributed_value(
        &self,
    ) -> Result<Option<Retained<NSAttributedString>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedValue))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityAttributedValue(self.main_thread))
    }

    /// Replace UIKit's `accessibilityLanguage` property with a caller-supplied BCP 47 tag.
    ///
    /// `None` clears the property. The iOS SDK declares no explicit minimum version for this
    /// property; its selector is checked before dispatch. The tag is passed through unchanged and
    /// is not validated or normalized.
    pub fn set_accessibility_language(
        &self,
        language_tag: Option<&str>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityLanguage:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let language_tag = map_optional_text(language_tag, NSString::from_str);
        self.view
            .setAccessibilityLanguage(language_tag.as_deref(), self.main_thread);
        Ok(())
    }

    /// Read UIKit's nullable `accessibilityLanguage` property as an owned Rust string.
    ///
    /// The property getter is selector-checked and its text is copied without validation or
    /// normalization. This does not invoke `accessibilityLanguageBlock` or report spoken output.
    pub fn accessibility_language(&self) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityLanguage)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityLanguage(self.main_thread)
            .map(|language| language.to_string()))
    }

    /// Replace UIKit's `accessibilityUserInputLabels` property with caller-owned alternatives.
    ///
    /// The primary label should be first, with alternatives in descending importance. An empty
    /// slice assigns an empty array. The iOS 13.0 property is selector-checked; labels pass through
    /// without validation or normalization.
    pub fn set_accessibility_user_input_labels(
        &self,
        labels: &[&str],
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityUserInputLabels:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let native_labels = labels
            .iter()
            .map(|label| NSString::from_str(label))
            .collect::<Vec<_>>();
        let native_labels = NSArray::from_retained_slice(&native_labels);
        // SAFETY: The setter receives a non-null `NSArray<NSString>` built above. The generated
        // binding marks `None` unsafe because the native property may not accept nil; this call
        // never passes `None`, and every array element is an NSString.
        unsafe {
            self.view
                .setAccessibilityUserInputLabels(Some(&native_labels), self.main_thread);
        }
        Ok(())
    }

    /// Read UIKit's current plain user-input labels as Rust-owned strings in native order.
    ///
    /// The iOS 13.0 getter is selector-checked. UIKit controls may provide defaults, and this reads
    /// the property only; it does not invoke `accessibilityUserInputLabelsBlock` or normalize text.
    pub fn accessibility_user_input_labels(
        &self,
    ) -> Result<Option<Vec<String>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityUserInputLabels))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let Some(native_labels) = self.view.accessibilityUserInputLabels(self.main_thread) else {
            return Ok(None);
        };
        let mut labels = Vec::with_capacity(native_labels.count());
        for index in 0..native_labels.count() {
            labels.push(native_labels.objectAtIndex(index).to_string());
        }
        Ok(Some(labels))
    }

    /// Replace UIKit's `accessibilityAttributedUserInputLabels` with caller-ordered labels.
    ///
    /// Put the primary label first and alternatives in descending importance. An empty slice
    /// assigns an empty array. The iOS 13.0 property is selector-checked; UIKit copies the array.
    pub fn set_accessibility_attributed_user_input_labels(
        &self,
        labels: &[&NSAttributedString],
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityAttributedUserInputLabels:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let native_labels = NSArray::from_slice(labels);
        self.view
            .setAccessibilityAttributedUserInputLabels(Some(&native_labels), self.main_thread);
        Ok(())
    }

    /// Read UIKit's current attributed user-input labels as owned retained values in native order.
    ///
    /// The iOS 13.0 getter is selector-checked. UIKit controls may supply defaults; the getter
    /// does not invoke `accessibilityAttributedUserInputLabelsBlock` or claim speech recognition.
    pub fn accessibility_attributed_user_input_labels(
        &self,
    ) -> Result<Vec<Retained<NSAttributedString>>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedUserInputLabels))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let native_labels = self
            .view
            .accessibilityAttributedUserInputLabels(self.main_thread);
        let mut labels = Vec::with_capacity(native_labels.count());
        for index in 0..native_labels.count() {
            labels.push(native_labels.objectAtIndex(index));
        }
        Ok(labels)
    }

    /// Replace UIKit's `accessibilityTextualContext` property.
    ///
    /// `None` clears the property. The iOS 13.0 property is checked with `respondsToSelector:`
    /// first. Only this crate's named public UIKit constants can be assigned.
    pub fn set_accessibility_textual_context(
        &self,
        context: Option<AccessibilityTextualContext>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityTextualContext:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let context = context.map(native_textual_context);
        self.view
            .setAccessibilityTextualContext(context, self.main_thread);
        Ok(())
    }

    /// Read UIKit's nullable `accessibilityTextualContext` property as an owned Rust string.
    ///
    /// The iOS 13.0 getter is selector-checked. The value is copied without mapping, validation, or
    /// normalization; this does not invoke `accessibilityTextualContextBlock` or claim spoken output.
    pub fn accessibility_textual_context(
        &self,
    ) -> Result<Option<String>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityTextualContext))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityTextualContext(self.main_thread)
            .map(|context| context.to_string()))
    }

    /// Return whether UIKit's current label getter returns a non-nil string.
    ///
    /// An empty string counts as present. A UIKit control may supply a default label; this does not
    /// report whether the caller explicitly set one.
    pub fn has_label(&self) -> bool {
        self.view.accessibilityLabel(self.main_thread).is_some()
    }

    /// Return whether UIKit's current attributed accessibility-label property is non-nil.
    ///
    /// The iOS 11.0 getter is selector-checked. This reads the property only and does not invoke
    /// the iOS 17 `accessibilityAttributedLabelBlock` or report assistive-application output.
    pub fn has_accessibility_attributed_label(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedLabel))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityAttributedLabel(self.main_thread)
            .is_some())
    }

    /// Return whether UIKit's current hint getter returns a non-nil string.
    ///
    /// An empty string counts as present. A UIKit control may supply a default value; this does not
    /// report whether the caller explicitly set one.
    pub fn has_hint(&self) -> bool {
        self.view.accessibilityHint(self.main_thread).is_some()
    }

    /// Return whether UIKit's current attributed accessibility-hint property is non-nil.
    ///
    /// The iOS 11.0 getter is selector-checked. This reads the property only and does not invoke
    /// the iOS 17 `accessibilityAttributedHintBlock` or report assistive-application output.
    pub fn has_accessibility_attributed_hint(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedHint))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityAttributedHint(self.main_thread)
            .is_some())
    }

    /// Return whether UIKit's current value getter returns a non-nil string.
    ///
    /// An empty string counts as present. This reports the property getter only, not a dynamic
    /// accessibility callback.
    pub fn has_value(&self) -> bool {
        self.view.accessibilityValue(self.main_thread).is_some()
    }

    /// Return whether UIKit's current attributed accessibility-value property is non-nil.
    ///
    /// The iOS 11.0 getter is selector-checked. This reads the property only and does not invoke
    /// the iOS 17 `accessibilityAttributedValueBlock` or report assistive-application output.
    pub fn has_accessibility_attributed_value(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedValue))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityAttributedValue(self.main_thread)
            .is_some())
    }

    /// Return whether UIKit's current `accessibilityLanguage` property is non-nil.
    ///
    /// The property getter is checked with `respondsToSelector:` first. This does not invoke an
    /// `accessibilityLanguageBlock` or report the language an assistive application uses.
    pub fn has_accessibility_language(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityLanguage)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityLanguage(self.main_thread).is_some())
    }

    /// Return whether UIKit's current user-input-label array is non-empty.
    ///
    /// The iOS 13.0 getter is selector-checked. UIKit controls may provide a default label, and
    /// this does not invoke `accessibilityUserInputLabelsBlock` or prove the caller set labels.
    pub fn has_accessibility_user_input_labels(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityUserInputLabels))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityUserInputLabels(self.main_thread)
            .is_some_and(|labels| !labels.is_empty()))
    }

    /// Return whether UIKit's attributed user-input-label array is non-empty.
    ///
    /// The iOS 13.0 getter is selector-checked. This reads the property only and does not invoke
    /// the iOS 17 `accessibilityAttributedUserInputLabelsBlock` or prove the caller set labels.
    pub fn has_accessibility_attributed_user_input_labels(
        &self,
    ) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityAttributedUserInputLabels))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(!self
            .view
            .accessibilityAttributedUserInputLabels(self.main_thread)
            .is_empty())
    }

    /// Read UIKit's current accessibility activation point in screen coordinates.
    ///
    /// UIKit defaults this to the midpoint of `accessibilityFrame`. The iOS 5.0 property is
    /// selector-checked; this returns the native point only and does not report an activation
    /// result.
    pub fn accessibility_activation_point(&self) -> Result<CGPoint, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityActivationPoint))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityActivationPoint(self.main_thread))
    }

    /// Replace UIKit's accessibility activation point with a caller-supplied screen point.
    ///
    /// The iOS 5.0 property is selector-checked. The point is passed through unchanged; callers
    /// must recompute it when layout or screen position changes. This does not convert coordinate
    /// spaces or guarantee an assistive application's activation behavior.
    pub fn set_accessibility_activation_point(
        &self,
        point: CGPoint,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityActivationPoint:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        self.view
            .setAccessibilityActivationPoint(point, self.main_thread);
        Ok(())
    }

    /// Return UIKit's current accessibility frame in screen coordinates.
    ///
    /// The property is selector-checked. The `UIView` default is its frame; this reads the native
    /// rectangle only and does not report visibility or assistive-application behavior.
    pub fn accessibility_frame(&self) -> Result<CGRect, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityFrame)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityFrame(self.main_thread))
    }

    /// Convert a rectangle from this borrowed view's coordinate space to screen coordinates.
    ///
    /// Use the result as a screen-space `accessibilityFrame` value. The adapter's main-thread
    /// proof is required, and callers must call only on iOS 7.0 or later. The generated direct C
    /// import has no runtime availability or weak-link guard. This method does not mutate the view
    /// or promise visibility or presentation.
    pub fn convert_frame_to_screen_coordinates(&self, frame_in_view: CGRect) -> CGRect {
        UIAccessibilityConvertFrameToScreenCoordinates(frame_in_view, self.view)
    }

    /// Replace UIKit's accessibility frame with a caller-supplied screen-space rectangle.
    ///
    /// The rectangle is passed through unchanged; this method does not convert coordinates, alter
    /// the view's layout frame, or guarantee assistive-application behavior.
    pub fn set_accessibility_frame(
        &self,
        frame: CGRect,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(setAccessibilityFrame:)) {
            return Err(AccessibilityApiUnavailable);
        }
        self.view.setAccessibilityFrame(frame, self.main_thread);
        Ok(())
    }

    /// Replace UIKit's accessibility path; `None` clears the property.
    ///
    /// The iOS 7.0 property is selector-checked. UIKit copies the supplied path, which must use
    /// screen coordinates. This method performs no coordinate conversion and makes no claim about
    /// assistive-application highlighting or activation behavior.
    pub fn set_accessibility_path(
        &self,
        path: Option<&UIBezierPath>,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(setAccessibilityPath:)) {
            return Err(AccessibilityApiUnavailable);
        }
        self.view.setAccessibilityPath(path, self.main_thread);
        Ok(())
    }

    /// Convert a path from this borrowed view's coordinate space to screen coordinates.
    ///
    /// This returns a new path and leaves the input path and view unchanged. The adapter's
    /// main-thread proof is required, and callers must call only on iOS 7.0 or later. The generated
    /// direct C import has no runtime availability or weak-link guard. This does not render the
    /// path or promise visibility or presentation.
    pub fn convert_path_to_screen_coordinates(
        &self,
        path_in_view: &UIBezierPath,
    ) -> Retained<UIBezierPath> {
        UIAccessibilityConvertPathToScreenCoordinates(path_in_view, self.view)
    }

    /// Return whether UIKit's current accessibility-path property is non-nil.
    ///
    /// The iOS 7.0 getter is selector-checked. This reads the property only and does not invoke
    /// the iOS 17 `accessibilityPathBlock` or report assistive-application behavior.
    pub fn has_accessibility_path(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityPath)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityPath(self.main_thread).is_some())
    }

    /// Return an owned copy of UIKit's current accessibility path, preserving a native `nil`.
    ///
    /// The iOS 7.0 getter is selector-checked. UIKit stores a copied path in screen coordinates;
    /// this method copies the returned mutable path so callers cannot mutate UIKit's stored value.
    /// It does not convert coordinates, invoke the iOS 17 `accessibilityPathBlock`, or report
    /// assistive-application behavior.
    pub fn accessibility_path(
        &self,
    ) -> Result<Option<Retained<UIBezierPath>>, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityPath)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityPath(self.main_thread)
            .map(|path| path.copy()))
    }

    /// Return whether UIKit's current `accessibilityTextualContext` property is non-nil.
    ///
    /// The iOS 13.0 property getter is checked with `respondsToSelector:` first. This does not
    /// invoke the iOS 17 `accessibilityTextualContextBlock` or report an assistive application's
    /// chosen output.
    pub fn has_accessibility_textual_context(&self) -> Result<bool, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityTextualContext))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self
            .view
            .accessibilityTextualContext(self.main_thread)
            .is_some())
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
    /// If the set includes `AccessibilityTrait::TabBar`, the caller must use the iOS 10.0 trait
    /// only for a view that represents an ordered tab list and set that view's element flag to
    /// `false`; this setter does not create or order tab children.
    pub fn set_traits(&self, traits: &[AccessibilityTrait]) {
        // SAFETY: `UIAccessibilityTraitNone` is a public immutable UIKit constant from the active
        // SDK's `UIAccessibilityConstants.h` and is linked from UIKit.
        let mask = fold_traits(traits, unsafe { UIAccessibilityTraitNone }, native_trait);
        self.view.setAccessibilityTraits(mask, self.main_thread);
    }

    /// Check whether UIKit's current trait mask contains the selected framework-owned trait.
    ///
    /// This includes traits supplied by UIKit defaults. Unknown native trait flags remain hidden;
    /// this method returns no raw mask or assistive-technology behavior claim.
    pub fn has_trait(&self, trait_: AccessibilityTrait) -> bool {
        let requested = native_trait(trait_);
        let current = self.view.accessibilityTraits(self.main_thread);
        requested != 0 && current & requested == requested
    }

    /// Read UIKit's current raw accessibility-traits bit mask.
    ///
    /// The property is selector-checked and returned unchanged, including flags this crate does
    /// not name. UIKit defaults may contribute flags; this does not invoke the iOS 17
    /// `accessibilityTraitsBlock` or report assistive-application behavior.
    pub fn accessibility_traits_mask(&self) -> Result<u64, AccessibilityApiUnavailable> {
        if !self.view.respondsToSelector(sel!(accessibilityTraits)) {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityTraits(self.main_thread))
    }

    /// Read UIKit's current direct-touch option mask without interpreting or changing it.
    ///
    /// The iOS 17.0 property is selector-checked and returned as the generated typed options
    /// value, preserving flags this crate does not name. This does not detect VoiceOver state,
    /// enable direct touch, or guarantee touch delivery or audio behavior.
    pub fn accessibility_direct_touch_options(
        &self,
    ) -> Result<UIAccessibilityDirectTouchOptions, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityDirectTouchOptions))
        {
            return Err(AccessibilityApiUnavailable);
        }
        Ok(self.view.accessibilityDirectTouchOptions(self.main_thread))
    }

    /// Read UIKit's current `accessibilityNavigationStyle` property.
    ///
    /// The iOS 8.0 property is checked with `respondsToSelector:` first. `None` means UIKit returned
    /// a value outside this crate's three named styles. The SDK documents this property as
    /// affecting Switch Control only, not VoiceOver or other assistive technologies.
    pub fn accessibility_navigation_style(
        &self,
    ) -> Result<Option<AccessibilityNavigationStyle>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityNavigationStyle))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let style = self.view.accessibilityNavigationStyle(self.main_thread);
        Ok(match style {
            NativeAccessibilityNavigationStyle::Automatic => {
                Some(AccessibilityNavigationStyle::Automatic)
            }
            NativeAccessibilityNavigationStyle::Separate => {
                Some(AccessibilityNavigationStyle::Separate)
            }
            NativeAccessibilityNavigationStyle::Combined => {
                Some(AccessibilityNavigationStyle::Combined)
            }
            _ => None,
        })
    }

    /// Replace UIKit's `accessibilityNavigationStyle` property with one of its public values.
    ///
    /// The iOS 8.0 setter is checked with `respondsToSelector:` first. UIKit documents this
    /// property as affecting Switch Control only, not VoiceOver or other assistive technologies.
    /// This sets metadata only and does not establish an assistive-technology navigation result.
    pub fn set_accessibility_navigation_style(
        &self,
        style: AccessibilityNavigationStyle,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityNavigationStyle:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let style = match style {
            AccessibilityNavigationStyle::Automatic => {
                NativeAccessibilityNavigationStyle::Automatic
            }
            AccessibilityNavigationStyle::Separate => NativeAccessibilityNavigationStyle::Separate,
            AccessibilityNavigationStyle::Combined => NativeAccessibilityNavigationStyle::Combined,
        };
        self.view
            .setAccessibilityNavigationStyle(style, self.main_thread);
        Ok(())
    }

    /// Read UIKit's current `accessibilityContainerType` property.
    ///
    /// The iOS 11.0 container property is checked with `respondsToSelector:` first. `None` means
    /// UIKit returned a native value outside this adapter's supported subset.
    pub fn accessibility_container_type(
        &self,
    ) -> Result<Option<AccessibilityContainerType>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityContainerType))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let container_type = self.view.accessibilityContainerType(self.main_thread);
        Ok(match container_type {
            NativeAccessibilityContainerType::None => Some(AccessibilityContainerType::Unspecified),
            NativeAccessibilityContainerType::List => Some(AccessibilityContainerType::List),
            NativeAccessibilityContainerType::Landmark => {
                Some(AccessibilityContainerType::Landmark)
            }
            NativeAccessibilityContainerType::SemanticGroup => {
                Some(AccessibilityContainerType::SemanticGroup)
            }
            _ => None,
        })
    }

    /// Replace UIKit's `accessibilityContainerType` with a supported container type.
    ///
    /// The iOS 11.0 property is checked with `respondsToSelector:` first. This changes metadata
    /// only and does not create or enumerate the receiver's accessible children.
    pub fn set_accessibility_container_type(
        &self,
        container_type: AccessibilityContainerType,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityContainerType:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let container_type = match container_type {
            AccessibilityContainerType::Unspecified => NativeAccessibilityContainerType::None,
            AccessibilityContainerType::List => NativeAccessibilityContainerType::List,
            AccessibilityContainerType::Landmark => NativeAccessibilityContainerType::Landmark,
            AccessibilityContainerType::SemanticGroup => {
                NativeAccessibilityContainerType::SemanticGroup
            }
        };
        self.view
            .setAccessibilityContainerType(container_type, self.main_thread);
        Ok(())
    }

    /// Read UIKit's scalar `accessibilityExpandedStatus` property.
    ///
    /// The iOS 18.0 property is selector-checked. The result maps only the three known values;
    /// unknown native values return `None`. This does not invoke the iOS 18
    /// `accessibilityExpandedStatusBlock` or report an assistive application's output.
    pub fn accessibility_expanded_status(
        &self,
    ) -> Result<Option<AccessibilityExpandedStatus>, AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(accessibilityExpandedStatus))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let status = self.view.accessibilityExpandedStatus(self.main_thread);
        Ok(match status {
            NativeAccessibilityExpandedStatus::Unsupported => {
                Some(AccessibilityExpandedStatus::Unsupported)
            }
            NativeAccessibilityExpandedStatus::Expanded => {
                Some(AccessibilityExpandedStatus::Expanded)
            }
            NativeAccessibilityExpandedStatus::Collapsed => {
                Some(AccessibilityExpandedStatus::Collapsed)
            }
            _ => None,
        })
    }

    /// Replace UIKit's scalar `accessibilityExpandedStatus` property.
    ///
    /// The iOS 18.0 property is selector-checked. This sets metadata only and does not expand or
    /// collapse content. The iOS 18 `accessibilityExpandedStatusBlock` can take precedence; this
    /// method does not call or replace that block. The caller owns state accuracy.
    pub fn set_accessibility_expanded_status(
        &self,
        status: AccessibilityExpandedStatus,
    ) -> Result<(), AccessibilityApiUnavailable> {
        if !self
            .view
            .respondsToSelector(sel!(setAccessibilityExpandedStatus:))
        {
            return Err(AccessibilityApiUnavailable);
        }
        let status = match status {
            AccessibilityExpandedStatus::Unsupported => {
                NativeAccessibilityExpandedStatus::Unsupported
            }
            AccessibilityExpandedStatus::Expanded => NativeAccessibilityExpandedStatus::Expanded,
            AccessibilityExpandedStatus::Collapsed => NativeAccessibilityExpandedStatus::Collapsed,
        };
        self.view
            .setAccessibilityExpandedStatus(status, self.main_thread);
        Ok(())
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
            AccessibilityTrait::TabBar => UIAccessibilityTraitTabBar,
        }
    }
}

fn native_textual_context(
    context: AccessibilityTextualContext,
) -> &'static NativeAccessibilityTextualContext {
    // SAFETY: Each value is an immutable public UIKit textual-context NSString constant generated
    // by objc2-ui-kit from the active SDK's UIAccessibilityConstants.h.
    unsafe {
        match context {
            AccessibilityTextualContext::WordProcessing => {
                UIAccessibilityTextualContextWordProcessing
            }
            AccessibilityTextualContext::Narrative => UIAccessibilityTextualContextNarrative,
            AccessibilityTextualContext::Messaging => UIAccessibilityTextualContextMessaging,
            AccessibilityTextualContext::Spreadsheet => UIAccessibilityTextualContextSpreadsheet,
            AccessibilityTextualContext::FileSystem => UIAccessibilityTextualContextFileSystem,
            AccessibilityTextualContext::SourceCode => UIAccessibilityTextualContextSourceCode,
            AccessibilityTextualContext::Console => UIAccessibilityTextualContextConsole,
        }
    }
}
