use core::marker::PhantomData;
use ios_runtime::main_thread::MainThread;
use objc2::runtime::NSObjectProtocol;
use objc2::{MainThreadMarker, sel};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{NSArray, NSAttributedString, NSString};
use objc2_ui_kit::{
    NSObjectUIAccessibility, NSObjectUIAccessibilityContainer,
    UIAccessibilityContainerType as NativeAccessibilityContainerType,
    UIAccessibilityNavigationStyle as NativeAccessibilityNavigationStyle,
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
    UIAccessibilityTraitSummaryElement, UIAccessibilityTraitUpdatesFrequently,
    UIAccessibilityTraits, UIView,
};

use crate::container_type::AccessibilityContainerType;
use crate::navigation_style::AccessibilityNavigationStyle;
use crate::text::map_optional_text;
use crate::textual_context::AccessibilityTextualContext;
use crate::traits::{AccessibilityTrait, fold_traits};

/// The borrowed UIKit view does not respond to an optional accessibility property selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessibilityApiUnavailable;

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

    /// Replace whether UIKit exposes this view as an accessibility element.
    pub fn set_element(&self, is_element: bool) {
        self.view
            .setIsAccessibilityElement(is_element, self.main_thread);
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

    /// Replace the hint; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_hint(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityHint(value.as_deref(), self.main_thread);
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

    /// Replace the value; `None` clears it and `Some("")` assigns an empty string.
    pub fn set_value(&self, value: Option<&str>) {
        let value = map_optional_text(value, NSString::from_str);
        self.view
            .setAccessibilityValue(value.as_deref(), self.main_thread);
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
        };
        self.view
            .setAccessibilityContainerType(container_type, self.main_thread);
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
