use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use framework_ui::Frame;
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_ui_kit::{
    NSLayoutConstraint, NSObjectUIAccessibility, UIButton, UIControlContentHorizontalAlignment,
    UIControlEvents, UIControlState, UILayoutConstraintAxis, UIStackView, UIView,
};

use crate::disclosure_policy::{
    DISCLOSURE_HEADER_HEIGHT, content_height, update_expanded, visible_height,
};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

struct DisclosureGroupTargetState {
    expanded: Cell<bool>,
    action: RefCell<Box<dyn FnMut(bool)>>,
    content: Retained<UIView>,
    total_height: f64,
    label: Retained<NSString>,
    collapsed_title: Retained<NSString>,
    expanded_title: Retained<NSString>,
    collapsed_value: Retained<NSString>,
    expanded_value: Retained<NSString>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - DisclosureGroupTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiDisclosureGroupTarget"]
    #[ivars = DisclosureGroupTargetState]
    struct DisclosureGroupTarget;

    impl DisclosureGroupTarget {
        #[unsafe(method(disclosureGroupTapped:))]
        fn disclosure_group_tapped(&self, sender: &UIButton) {
            let _keep_alive = self.retain();
            let expanded = !self.ivars().expanded.get();
            if let Ok(marker) = main_thread_marker() {
                update_expanded(&self.ivars().expanded, expanded);
                apply_expanded_state(sender, &self.ivars().content, self.ivars(), expanded, marker);
                super::super::toggle_policy::invoke_toggle_callback(
                    &self.ivars().action,
                    expanded,
                );
            }
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for DisclosureGroupTarget {}
);

impl DisclosureGroupTarget {
    fn with_state(mtm: MainThreadMarker, state: DisclosureGroupTargetState) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(state);
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

/// A UIKit disclosure group with explicit expanded state and caller-owned native content.
///
/// The vertical `UIStackView` owns a 44-point button header and a content host sized from the
/// expanded frame. Keep this handle alive while its callback should remain active. Its native
/// objects and callback are main-thread-only; dropping the handle unregisters its target/action.
pub struct IosDisclosureGroup {
    native: Retained<UIStackView>,
    button: Retained<UIButton>,
    content: IosView,
    target: Retained<DisclosureGroupTarget>,
    _header_height: Retained<NSLayoutConstraint>,
    _content_height: Retained<NSLayoutConstraint>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a caller-owned-content disclosure group to `parent`.
    ///
    /// `frame` is the expanded group frame in parent-local UIKit points and must provide at least
    /// 44 points for the header. Arrange `native_stack_view()` in a caller-owned vertical stack
    /// when following content must shift as the group collapses. Programmatic state updates do not
    /// invoke `action`; user taps report the committed expanded state synchronously.
    pub fn add_disclosure_group<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        title: &str,
        initially_expanded: bool,
        action: F,
    ) -> Result<IosDisclosureGroup>
    where
        F: FnMut(bool) + 'static,
    {
        let content_height = content_height(frame)?;
        let marker = main_thread_marker()?;
        let native = UIStackView::new(marker);
        let mut initial_frame = native_frame(frame);
        initial_frame.size.height = visible_height(initially_expanded, frame.height());
        native.setFrame(initial_frame);
        native.setAxis(UILayoutConstraintAxis::Vertical);
        native.setSpacing(0.0);

        let button = UIButton::new(marker);
        button.setContentHorizontalAlignment(UIControlContentHorizontalAlignment::Leading);
        let content = UIView::new(marker);
        content.setTranslatesAutoresizingMaskIntoConstraints(false);

        let header_height = button
            .heightAnchor()
            .constraintEqualToConstant(DISCLOSURE_HEADER_HEIGHT);
        header_height.setActive(true);
        let content_height_constraint = content
            .heightAnchor()
            .constraintEqualToConstant(content_height);
        content_height_constraint.setActive(true);

        let label = NSString::from_str(title);
        let collapsed_title = NSString::from_str(&format!("▸ {title}"));
        let expanded_title = NSString::from_str(&format!("▾ {title}"));
        let collapsed_value = NSString::from_str("Collapsed");
        let expanded_value = NSString::from_str("Expanded");
        let target = DisclosureGroupTarget::with_state(
            marker,
            DisclosureGroupTargetState {
                expanded: Cell::new(initially_expanded),
                action: RefCell::new(Box::new(action)),
                content: content.retain(),
                total_height: frame.height(),
                label: label.retain(),
                collapsed_title,
                expanded_title,
                collapsed_value,
                expanded_value,
            },
        );

        autoreleasepool(|_| {
            button.setTitle_forState(
                Some(if initially_expanded {
                    &target.ivars().expanded_title
                } else {
                    &target.ivars().collapsed_title
                }),
                UIControlState::Normal,
            );
            button.setAccessibilityLabel(Some(&target.ivars().label), marker);
            button.setAccessibilityValue(
                Some(if initially_expanded {
                    &target.ivars().expanded_value
                } else {
                    &target.ivars().collapsed_value
                }),
                marker,
            );
        });
        content.setHidden(!initially_expanded);

        native.addArrangedSubview(&button);
        native.addArrangedSubview(&content);
        // SAFETY: `target` remains strongly owned by `IosDisclosureGroup` and implements the
        // exact selector registered for this button's TouchUpInside event.
        unsafe {
            button.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(disclosureGroupTapped:),
                UIControlEvents::TouchUpInside,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosDisclosureGroup {
            native,
            button,
            content: IosView {
                native: content,
                _not_send: PhantomData,
            },
            target,
            _header_height: header_height,
            _content_height: content_height_constraint,
            _not_send: PhantomData,
        })
    }
}

impl IosDisclosureGroup {
    /// Returns the current state on UIKit's main thread.
    pub fn is_expanded(&self) -> Result<bool> {
        let _disclosure_state_thread = main_thread_marker()?;
        Ok(self.target.ivars().expanded.get())
    }

    /// Updates content visibility and header state without invoking the user-tap callback.
    pub fn set_expanded(&mut self, expanded: bool) -> Result<()> {
        let marker = main_thread_marker()?;
        if update_expanded(&self.target.ivars().expanded, expanded) {
            apply_expanded_state(
                &self.button,
                &self.content.native,
                self.target.ivars(),
                expanded,
                marker,
            );
        }
        Ok(())
    }

    /// Borrows the content host for caller-owned labels and controls.
    pub fn content_view_mut(&mut self) -> &mut IosView {
        &mut self.content
    }

    /// Borrows the native vertical stack view for caller-owned UIKit layout.
    pub fn native_stack_view(&self) -> &UIStackView {
        &self.native
    }

    /// Borrows the native disclosure button for caller-owned UIKit integration.
    pub fn native_button(&self) -> &UIButton {
        &self.button
    }
}

fn apply_expanded_state(
    button: &UIButton,
    content: &UIView,
    state: &DisclosureGroupTargetState,
    expanded: bool,
    marker: MainThreadMarker,
) {
    content.setHidden(!expanded);
    let title = if expanded {
        &state.expanded_title
    } else {
        &state.collapsed_title
    };
    let value = if expanded {
        &state.expanded_value
    } else {
        &state.collapsed_value
    };
    autoreleasepool(|_| {
        button.setTitle_forState(Some(title), UIControlState::Normal);
        button.setAccessibilityValue(Some(value), marker);
    });
    if let Some(group) = button.superview() {
        let mut frame = group.frame();
        frame.size.height = visible_height(expanded, state.total_height);
        group.setFrame(frame);
    }
}

impl Drop for IosDisclosureGroup {
    fn drop(&mut self) {
        // SAFETY: this main-thread-only handle removes only its retained target, selector, and
        // TouchUpInside registration before releasing callback ownership.
        unsafe {
            self.button.removeTarget_action_forControlEvents(
                Some(&*self.target),
                Some(sel!(disclosureGroupTapped:)),
                UIControlEvents::TouchUpInside,
            );
        }
    }
}
