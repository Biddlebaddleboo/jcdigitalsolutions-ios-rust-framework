//! A caller-supplied UIKit semantic tree for custom-rendered content.

use core::cell::RefCell;
use std::ptr;
use std::rc::{Rc, Weak};
use std::vec::Vec;

use ios_accessibility::AccessibilityTrait;
use ios_runtime::ffi::catch_unwind;
use objc2::rc::{Retained, Weak as NativeWeak};
use objc2::runtime::{AnyObject, NSObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{NSArray, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    NSObjectUIAccessibility, NSObjectUIAccessibilityAction, NSObjectUIAccessibilityContainer,
    UIAccessibilityCustomAction, UIAccessibilityElement, UIAccessibilityLayoutChangedNotification,
    UIAccessibilityPageScrolledNotification, UIAccessibilityPostNotification,
    UIAccessibilityScreenChangedNotification, UIAccessibilityTraitAdjustable,
    UIAccessibilityTraitAllowsDirectInteraction, UIAccessibilityTraitButton,
    UIAccessibilityTraitCausesPageTurn, UIAccessibilityTraitHeader, UIAccessibilityTraitImage,
    UIAccessibilityTraitKeyboardKey, UIAccessibilityTraitLink, UIAccessibilityTraitNone,
    UIAccessibilityTraitNotEnabled, UIAccessibilityTraitPlaysSound,
    UIAccessibilityTraitSearchField, UIAccessibilityTraitSelected,
    UIAccessibilityTraitStartsMediaSession, UIAccessibilityTraitStaticText,
    UIAccessibilityTraitSummaryElement, UIAccessibilityTraitTabBar,
    UIAccessibilityTraitUpdatesFrequently, UIAccessibilityTraits, UIImageView,
};

use crate::custom_render_accessibility_policy::{
    AccessibilityActionEvent, AccessibilityActionId, AccessibilityNodeId, SemanticActionKind,
    SemanticActionShape, SemanticFrame, SemanticNodeShape, SemanticTreeError, SemanticTreeState,
    validate_nodes,
};
use crate::custom_render_host::IosCustomRenderHost;

/// One localized action that a semantic node exposes to assistive technology.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticAccessibilityAction {
    id: AccessibilityActionId,
    label: String,
}

impl SemanticAccessibilityAction {
    /// Create an action with a stable, node-local identity and a nonempty spoken label.
    pub fn new(id: AccessibilityActionId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Return the stable action identity.
    pub const fn id(&self) -> AccessibilityActionId {
        self.id
    }

    /// Return the label UIKit presents for this action.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// One custom-rendered semantic node in host-local UIKit points.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticAccessibilityNode {
    id: AccessibilityNodeId,
    frame: SemanticFrame,
    label: Option<String>,
    value: Option<String>,
    hint: Option<String>,
    traits: Vec<AccessibilityTrait>,
    activation: Option<SemanticAccessibilityAction>,
    custom_actions: Vec<SemanticAccessibilityAction>,
}

impl SemanticAccessibilityNode {
    /// Create one node with a stable identity and a host-local point-space frame.
    pub fn new(id: AccessibilityNodeId, frame: SemanticFrame) -> Self {
        Self {
            id,
            frame,
            label: None,
            value: None,
            hint: None,
            traits: Vec::new(),
            activation: None,
            custom_actions: Vec::new(),
        }
    }

    /// Set the spoken label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the current spoken value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the spoken hint.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Replace the standard UIKit traits associated with this node.
    pub fn with_traits(mut self, traits: impl IntoIterator<Item = AccessibilityTrait>) -> Self {
        self.traits = traits.into_iter().collect();
        self
    }

    /// Set the node's default activation action.
    ///
    /// On iOS 17 and later UIKit routes the default accessibility activation to this event. On
    /// iOS 12 through 16, the same action is exposed as a named custom action because UIKit's
    /// public activation-block property is only available starting in iOS 17.
    pub fn with_activation(mut self, action: SemanticAccessibilityAction) -> Self {
        self.activation = Some(action);
        self
    }

    /// Append one named custom action in VoiceOver rotor order.
    pub fn with_custom_action(mut self, action: SemanticAccessibilityAction) -> Self {
        self.custom_actions.push(action);
        self
    }

    /// Return the stable node identity.
    pub const fn id(&self) -> AccessibilityNodeId {
        self.id
    }
}

struct ActionTargetState {
    tree: Weak<RefCell<SemanticTreeState>>,
    event: AccessibilityActionEvent,
}

define_class!(
    // SAFETY: This NSObject target has no additional superclass invariants and no custom Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosCustomRenderAccessibilityActionTarget"]
    #[ivars = ActionTargetState]
    struct CustomRenderAccessibilityActionTarget;

    unsafe impl NSObjectProtocol for CustomRenderAccessibilityActionTarget {}

    impl CustomRenderAccessibilityActionTarget {
        #[allow(non_snake_case)]
        #[unsafe(method(performAccessibilityAction:))]
        fn perform_accessibility_action(&self, _action: &UIAccessibilityCustomAction) -> bool {
            catch_unwind(|| {
                let Some(tree) = self.ivars().tree.upgrade() else {
                    return false;
                };
                let Ok(mut state) = tree.try_borrow_mut() else {
                    return false;
                };
                let event = self.ivars().event;
                state.record_action(event.generation, event.node_id, event.action_id, event.kind)
            })
            .unwrap_or(false)
        }
    }
);

impl CustomRenderAccessibilityActionTarget {
    fn new(marker: MainThreadMarker, state: ActionTargetState) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(state);
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's initializer is valid.
        unsafe { msg_send![super(this), init] }
    }
}

struct NativeSemanticNode {
    id: AccessibilityNodeId,
    element: Retained<UIAccessibilityElement>,
    action_targets: Vec<Retained<CustomRenderAccessibilityActionTarget>>,
}

/// Main-thread adapter that maps an ordered Rust semantic tree to native UIKit accessibility elements.
///
/// It borrows the Phase A `UIImageView` through a weak Objective-C reference. The caller continues
/// to own widget semantics, hit testing, drawing, scrolling and activation policy. Action requests
/// enter a bounded queue and never call application closures from UIKit.
pub struct IosCustomRenderAccessibility {
    container: NativeWeak<UIImageView>,
    state: Rc<RefCell<SemanticTreeState>>,
    nodes: Vec<NativeSemanticNode>,
    original_is_accessibility_element: bool,
    marker: MainThreadMarker,
}

impl IosCustomRenderAccessibility {
    /// Create an empty semantic tree for one custom-render host on the UIKit main thread.
    ///
    /// This adapter requires an otherwise unclaimed `accessibilityElements` list. It marks the host as
    /// a non-element container while active and restores its prior `isAccessibilityElement` value on
    /// drop. Create at most one adapter for a host at a time.
    pub fn new(
        host: &IosCustomRenderHost,
        marker: MainThreadMarker,
    ) -> Result<Self, SemanticTreeError> {
        let native_view = host.native_view();
        if native_view.accessibilityElements(marker).is_some() {
            return Err(SemanticTreeError::ContainerAlreadyHasAccessibilityElements);
        }
        let original_is_accessibility_element = native_view.isAccessibilityElement(marker);
        native_view.setIsAccessibilityElement(false, marker);
        let empty_elements: Retained<NSArray<AnyObject>> = NSArray::from_slice(&[]);
        // SAFETY: The host list was just verified unclaimed; the empty array reserves it for this
        // adapter until Drop clears it and restores the host's element flag.
        unsafe { native_view.setAccessibilityElements(Some(&empty_elements), marker) };
        Ok(Self {
            container: NativeWeak::new(host.native_view()),
            state: Rc::new(RefCell::new(SemanticTreeState::new())),
            nodes: Vec::new(),
            original_is_accessibility_element,
            marker,
        })
    }

    /// Atomically replace the ordered semantic-node snapshot and return its generation.
    ///
    /// Duplicate node or per-node action IDs and nonfinite/negative frame extents are rejected
    /// before UIKit state changes. Elements with unchanged IDs are reused so UIKit retains their
    /// identity across label, geometry and order updates.
    pub fn set_nodes(
        &mut self,
        nodes: &[SemanticAccessibilityNode],
    ) -> Result<u32, SemanticTreeError> {
        let shapes: Vec<_> = nodes.iter().map(node_shape).collect();
        validate_nodes(&shapes)?;
        if nodes.iter().any(|node| {
            node.traits.iter().any(|trait_| {
                matches!(
                    trait_,
                    AccessibilityTrait::Adjustable | AccessibilityTrait::TabBar
                )
            }) || node
                .activation
                .iter()
                .chain(node.custom_actions.iter())
                .any(|action| action.label.is_empty())
        }) {
            return Err(SemanticTreeError::UnsupportedTraitOrAction);
        }
        let Some(container) = self.container.load() else {
            return Err(SemanticTreeError::HostUnavailable);
        };
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| SemanticTreeError::StateBusy)?;
        let generation = state.next_generation()?;

        let mut old_nodes: std::collections::BTreeMap<_, _> =
            self.nodes.drain(..).map(|node| (node.id, node)).collect();
        let mut next_nodes = Vec::with_capacity(nodes.len());
        let mut native_elements = Vec::with_capacity(nodes.len());
        for node in nodes {
            let mut native = old_nodes
                .remove(&node.id)
                .unwrap_or_else(|| NativeSemanticNode {
                    id: node.id,
                    element: unsafe {
                        UIAccessibilityElement::initWithAccessibilityContainer(
                            UIAccessibilityElement::alloc(self.marker),
                            &container as &AnyObject,
                        )
                    },
                    action_targets: Vec::new(),
                });
            clear_native_actions(&native.element, self.marker);
            native.action_targets.clear();
            apply_node_properties(&native.element, node);
            install_node_actions(
                &native.element,
                &mut native.action_targets,
                node,
                Rc::downgrade(&self.state),
                generation,
                self.marker,
            );
            native_elements.push(native.element.clone());
            next_nodes.push(native);
        }
        for removed in old_nodes.values() {
            clear_native_actions(&removed.element, self.marker);
        }
        let native_objects: Vec<&AnyObject> = native_elements
            .iter()
            .map(|element| element.as_ref() as &AnyObject)
            .collect();
        let native_array: Retained<NSArray<AnyObject>> = NSArray::from_slice(&native_objects);
        let generation = state.replace(&shapes)?;
        // SAFETY: Every object in the array is a UIAccessibilityElement created for this container.
        unsafe { container.setAccessibilityElements(Some(&native_array), self.marker) };
        self.nodes = next_nodes;
        drop(state);
        Ok(generation)
    }

    /// Return the next activation or custom-action request in native delivery order.
    pub fn take_action(&mut self) -> Option<AccessibilityActionEvent> {
        self.state.try_borrow_mut().ok()?.take_event()
    }

    /// Ask UIKit to move accessibility focus to a current semantic node.
    pub fn request_focus(&self, node_id: AccessibilityNodeId) -> bool {
        if !self
            .state
            .try_borrow()
            .is_ok_and(|state| state.contains_node(node_id))
        {
            return false;
        }
        let Some(node) = self.nodes.iter().find(|node| node.id == node_id) else {
            return false;
        };
        let element: &AnyObject = node.element.as_ref();
        // SAFETY: The notification and argument are public UIKit accessibility APIs; the retained
        // element remains alive for the duration of this synchronous call.
        unsafe {
            UIAccessibilityPostNotification(UIAccessibilityLayoutChangedNotification, Some(element))
        };
        true
    }

    /// Announce that the caller completed scrolling, using its localized page/status text.
    pub fn notify_scrolled(&self, announcement: &str) {
        let announcement = NSString::from_str(announcement);
        let argument: &AnyObject = announcement.as_ref();
        // SAFETY: The notification and NSString argument follow UIKit's public page-scrolled contract.
        unsafe {
            UIAccessibilityPostNotification(UIAccessibilityPageScrolledNotification, Some(argument))
        };
    }

    /// Ask UIKit to announce a screen change and optionally focus a current node.
    pub fn notify_screen_changed(&self, focus: Option<AccessibilityNodeId>) -> bool {
        let element = focus.and_then(|id| self.nodes.iter().find(|node| node.id == id));
        if focus.is_some() && element.is_none() {
            return false;
        }
        let argument = element.map(|node| node.element.as_ref() as &AnyObject);
        // SAFETY: The notification and optional element argument are public UIKit accessibility APIs.
        unsafe {
            UIAccessibilityPostNotification(UIAccessibilityScreenChangedNotification, argument)
        };
        true
    }
}

impl Drop for IosCustomRenderAccessibility {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.try_borrow_mut() {
            state.close();
        }
        if let Some(container) = self.container.load() {
            // SAFETY: This adapter exclusively owns the semantic element list it installed.
            unsafe { container.setAccessibilityElements(None, self.marker) };
        }
        for node in &self.nodes {
            clear_native_actions(&node.element, self.marker);
        }
        if let Some(container) = self.container.load() {
            container
                .setIsAccessibilityElement(self.original_is_accessibility_element, self.marker);
        }
    }
}

fn node_shape(node: &SemanticAccessibilityNode) -> SemanticNodeShape {
    SemanticNodeShape {
        id: node.id,
        frame: node.frame,
        actions: node
            .activation
            .iter()
            .map(|action| SemanticActionShape {
                id: action.id,
                kind: SemanticActionKind::Activate,
            })
            .chain(
                node.custom_actions
                    .iter()
                    .map(|action| SemanticActionShape {
                        id: action.id,
                        kind: SemanticActionKind::Custom,
                    }),
            )
            .collect(),
    }
}

fn apply_node_properties(element: &UIAccessibilityElement, node: &SemanticAccessibilityNode) {
    element.setIsAccessibilityElement(true);
    set_optional_text(element, &node.label, TextField::Label);
    set_optional_text(element, &node.value, TextField::Value);
    set_optional_text(element, &node.hint, TextField::Hint);
    let frame = CGRect::new(
        CGPoint::new(node.frame.x, node.frame.y),
        CGSize::new(node.frame.width, node.frame.height),
    );
    element.setAccessibilityFrameInContainerSpace(frame);
    // SAFETY: Each trait constant is a public, immutable UIKit value from the active SDK.
    let mut traits = unsafe { UIAccessibilityTraitNone };
    for trait_ in &node.traits {
        traits |= native_trait(*trait_);
    }
    element.setAccessibilityTraits(traits);
}

enum TextField {
    Label,
    Value,
    Hint,
}

fn set_optional_text(element: &UIAccessibilityElement, text: &Option<String>, field: TextField) {
    let native = text.as_deref().map(NSString::from_str);
    match field {
        TextField::Label => element.setAccessibilityLabel(native.as_deref()),
        TextField::Value => element.setAccessibilityValue(native.as_deref()),
        TextField::Hint => element.setAccessibilityHint(native.as_deref()),
    }
}

fn install_node_actions(
    element: &UIAccessibilityElement,
    targets: &mut Vec<Retained<CustomRenderAccessibilityActionTarget>>,
    node: &SemanticAccessibilityNode,
    state: Weak<RefCell<SemanticTreeState>>,
    generation: u32,
    marker: MainThreadMarker,
) {
    let mut native_actions = Vec::new();
    if let Some(activation) = &node.activation {
        let event = AccessibilityActionEvent {
            node_id: node.id,
            action_id: activation.id,
            kind: SemanticActionKind::Activate,
            generation,
        };
        if objc2::available!(ios = 17.0, ..) {
            let state = state.clone();
            let block: block2::RcBlock<dyn Fn() -> objc2::runtime::Bool> =
                block2::RcBlock::new(move || {
                    let success = catch_unwind(|| record_action(&state, event)).unwrap_or(false);
                    objc2::runtime::Bool::new(success)
                });
            // SAFETY: UIKit copies the main-thread block; it captures only a weak tree reference
            // and a generation-checked value event, so it cannot retain or call dropped app state.
            unsafe {
                element.setAccessibilityActivateBlock(block2::RcBlock::as_ptr(&block), marker)
            };
        } else {
            let (native_action, target) = create_selector_action(activation, event, &state, marker);
            native_actions.push(native_action);
            targets.push(target);
        }
    }
    for action in &node.custom_actions {
        let event = AccessibilityActionEvent {
            node_id: node.id,
            action_id: action.id,
            kind: SemanticActionKind::Custom,
            generation,
        };
        let (native_action, target) = create_selector_action(action, event, &state, marker);
        native_actions.push(native_action);
        targets.push(target);
    }
    if !native_actions.is_empty() {
        let native_actions = NSArray::from_retained_slice(&native_actions);
        element.setAccessibilityCustomActions(Some(&native_actions), marker);
    }
}

fn create_selector_action(
    action: &SemanticAccessibilityAction,
    event: AccessibilityActionEvent,
    state: &Weak<RefCell<SemanticTreeState>>,
    marker: MainThreadMarker,
) -> (
    Retained<UIAccessibilityCustomAction>,
    Retained<CustomRenderAccessibilityActionTarget>,
) {
    let target = CustomRenderAccessibilityActionTarget::new(
        marker,
        ActionTargetState {
            tree: state.clone(),
            event,
        },
    );
    let action_name = NSString::from_str(&action.label);
    let target_object: &AnyObject = target.as_ref();
    // SAFETY: The target is a live Rust-defined NSObject with this exact selector and signature;
    // its weak event-state reference and generation guard make retained stale actions inert.
    let native_action = unsafe {
        UIAccessibilityCustomAction::initWithName_target_selector(
            UIAccessibilityCustomAction::alloc(marker),
            &action_name,
            Some(target_object),
            sel!(performAccessibilityAction:),
        )
    };
    (native_action, target)
}

fn clear_native_actions(element: &UIAccessibilityElement, marker: MainThreadMarker) {
    if objc2::available!(ios = 17.0, ..) {
        // SAFETY: A null block clears the optional UIKit activation handler.
        unsafe { element.setAccessibilityActivateBlock(ptr::null_mut(), marker) };
    }
    element.setAccessibilityCustomActions(None, marker);
}

fn record_action(
    state: &Weak<RefCell<SemanticTreeState>>,
    event: AccessibilityActionEvent,
) -> bool {
    let Some(state) = state.upgrade() else {
        return false;
    };
    let Ok(mut state) = state.try_borrow_mut() else {
        return false;
    };
    state.record_action(event.generation, event.node_id, event.action_id, event.kind)
}

fn native_trait(trait_: AccessibilityTrait) -> UIAccessibilityTraits {
    match trait_ {
        AccessibilityTrait::Button => unsafe { UIAccessibilityTraitButton },
        AccessibilityTrait::Link => unsafe { UIAccessibilityTraitLink },
        AccessibilityTrait::Header => unsafe { UIAccessibilityTraitHeader },
        AccessibilityTrait::Image => unsafe { UIAccessibilityTraitImage },
        AccessibilityTrait::Selected => unsafe { UIAccessibilityTraitSelected },
        AccessibilityTrait::StaticText => unsafe { UIAccessibilityTraitStaticText },
        AccessibilityTrait::SearchField => unsafe { UIAccessibilityTraitSearchField },
        AccessibilityTrait::Adjustable => unsafe { UIAccessibilityTraitAdjustable },
        AccessibilityTrait::NotEnabled => unsafe { UIAccessibilityTraitNotEnabled },
        AccessibilityTrait::KeyboardKey => unsafe { UIAccessibilityTraitKeyboardKey },
        AccessibilityTrait::UpdatesFrequently => unsafe { UIAccessibilityTraitUpdatesFrequently },
        AccessibilityTrait::PlaysSound => unsafe { UIAccessibilityTraitPlaysSound },
        AccessibilityTrait::CausesPageTurn => unsafe { UIAccessibilityTraitCausesPageTurn },
        AccessibilityTrait::StartsMediaSession => unsafe { UIAccessibilityTraitStartsMediaSession },
        AccessibilityTrait::AllowsDirectInteraction => unsafe {
            UIAccessibilityTraitAllowsDirectInteraction
        },
        AccessibilityTrait::SummaryElement => unsafe { UIAccessibilityTraitSummaryElement },
        AccessibilityTrait::TabBar => unsafe { UIAccessibilityTraitTabBar },
    }
}
