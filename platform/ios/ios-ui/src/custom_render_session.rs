//! Per-root-view ownership and lifecycle composition for native custom rendering.

use core::fmt;
use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::{Error, ErrorKind, Result};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_core_foundation::CGRect;
use objc2_ui_kit::{UIImageView, UIView, UIViewAutoresizing};

use crate::custom_render_accessibility::{IosCustomRenderAccessibility, SemanticAccessibilityNode};
use crate::custom_render_accessibility_policy::{AccessibilityActionEvent, SemanticTreeError};
use crate::custom_render_host::{
    CustomRenderHostError, HostGeometry, IosCustomRenderHost, Rgba8Frame,
};
use crate::custom_render_session_policy::{
    PresentDecision, SessionEffect, SessionEvent, SessionPhase, present_decision, transition,
};
use crate::custom_render_text_input::{
    IosCustomTextInput, IosCustomTextInputSnapshot, IosTextInputTraits,
};
use crate::custom_render_touch::{IosCustomRenderTouchInput, IosTouchEvent, IosTouchInputFault};
use crate::display_link::{IosDisplayFrame, IosDisplayLink};

/// Failure to present through a session that is paused, detached, or cannot create its image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosCustomRenderSessionError {
    /// The session is suspended and does not accept new frames.
    Suspended,
    /// The session was detached and cannot be reused.
    Detached,
    /// UIKit or CoreGraphics could not create the submitted image.
    Host(CustomRenderHostError),
}

impl fmt::Display for IosCustomRenderSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "custom render session: {self:?}")
    }
}

impl std::error::Error for IosCustomRenderSessionError {}

impl From<CustomRenderHostError> for IosCustomRenderSessionError {
    fn from(error: CustomRenderHostError) -> Self {
        Self::Host(error)
    }
}

/// Own one custom-render host and its optional native input, accessibility, and display adapters.
///
/// The session retains one caller-provided root `UIView`, attaches its own image host to that
/// root, and keeps all native objects on the UIKit main thread. `suspend` pauses display callbacks,
/// cancels active touch sequences, resigns text focus, and disables owned input views. `detach` is
/// permanent and idempotent; queued touch cancellations remain available until this owner drops.
/// Frame data keeps the existing synchronous, single-copy semantics of [`IosCustomRenderHost`].
#[must_use = "dropping the session detaches its native host and adapters"]
pub struct IosCustomRenderSession {
    root_view: Retained<UIView>,
    render_host: IosCustomRenderHost,
    touch_input: Option<IosCustomRenderTouchInput>,
    text_input: Option<IosCustomTextInput>,
    accessibility: Option<IosCustomRenderAccessibility>,
    display_link: Option<IosDisplayLink>,
    phase: SessionPhase,
    marker: MainThreadMarker,
    _not_send_or_sync: PhantomData<Rc<()>>,
}

impl IosCustomRenderSession {
    /// Attach one full-bounds RGBA image host to an owned root view on the UIKit main thread.
    pub fn new(root_view: Retained<UIView>, marker: MainThreadMarker) -> Self {
        let bounds = root_view.bounds();
        let mut render_host = IosCustomRenderHost::new(marker);
        render_host.native_view().setAutoresizingMask(
            UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
        );
        render_host.attach_to(&root_view, bounds);
        Self {
            root_view,
            render_host,
            touch_input: None,
            text_input: None,
            accessibility: None,
            display_link: None,
            phase: SessionPhase::Active,
            marker,
            _not_send_or_sync: PhantomData,
        }
    }

    /// Borrow the retained root view for other UIKit-native controls or layout.
    pub fn root_view(&self) -> &UIView {
        &self.root_view
    }

    /// Borrow the session's native image view for UIKit-native integration.
    pub fn native_render_view(&self) -> &UIImageView {
        self.render_host.native_view()
    }

    /// Return the current host geometry and application state.
    pub fn geometry(&self) -> HostGeometry {
        self.render_host.geometry()
    }

    /// Set the custom-render host frame in root-view points.
    pub fn set_frame(&mut self, frame: CGRect) -> Result<()> {
        self.ensure_live()?;
        self.render_host.set_frame(frame);
        Ok(())
    }

    /// Copy and present one validated RGBA frame while the session is active.
    pub fn present(
        &self,
        frame: &Rgba8Frame<'_>,
    ) -> core::result::Result<(), IosCustomRenderSessionError> {
        match present_decision(self.phase) {
            PresentDecision::Allowed => self.render_host.present(frame).map_err(Into::into),
            PresentDecision::Suspended => Err(IosCustomRenderSessionError::Suspended),
            PresentDecision::Detached => Err(IosCustomRenderSessionError::Detached),
        }
    }

    /// Attach the bounded multi-touch overlay once for this session.
    pub fn enable_touch_input(&mut self) -> Result<()> {
        self.ensure_live()?;
        if self.touch_input.is_some() {
            return Err(Error::new(ErrorKind::AlreadyExists));
        }
        let input = IosCustomRenderTouchInput::attach(&self.render_host, self.marker);
        if self.phase == SessionPhase::Suspended {
            input.native_view().setUserInteractionEnabled(false);
        }
        self.touch_input = Some(input);
        Ok(())
    }

    /// Remove the oldest queued touch sample, including lifecycle cancellation samples.
    pub fn next_touch_event(&self) -> Option<IosTouchEvent> {
        self.touch_input
            .as_ref()
            .and_then(IosCustomRenderTouchInput::next_event)
    }

    /// Return the touch adapter's sticky overflow or invalid-sample fault, if present.
    pub fn touch_fault(&self) -> Option<IosTouchInputFault> {
        self.touch_input
            .as_ref()
            .and_then(IosCustomRenderTouchInput::fault)
    }

    /// Attach one native text-input surface to the retained root view.
    pub fn enable_text_input<F>(
        &mut self,
        frame: CGRect,
        initial_text: &str,
        traits: IosTextInputTraits,
        callback: F,
    ) -> Result<()>
    where
        F: FnMut(Result<IosCustomTextInputSnapshot>) + 'static,
    {
        self.ensure_live()?;
        if self.text_input.is_some() {
            return Err(Error::new(ErrorKind::AlreadyExists));
        }
        let input =
            IosCustomTextInput::attach(&self.root_view, frame, initial_text, traits, callback)?;
        if self.phase == SessionPhase::Suspended {
            input.native_text_view().setUserInteractionEnabled(false);
        }
        self.text_input = Some(input);
        Ok(())
    }

    /// Borrow the session's text-input handle for snapshots or native configuration.
    pub fn text_input(&self) -> Option<&IosCustomTextInput> {
        self.text_input.as_ref()
    }

    /// Borrow the session's text-input handle for updates.
    pub fn text_input_mut(&mut self) -> Option<&mut IosCustomTextInput> {
        self.text_input.as_mut()
    }

    /// Borrow the session's accessibility adapter for actions and UIKit announcements.
    pub fn accessibility_mut(&mut self) -> Option<&mut IosCustomRenderAccessibility> {
        self.accessibility.as_mut()
    }

    /// Install the semantic accessibility container once for this session.
    pub fn enable_accessibility(&mut self) -> core::result::Result<(), SemanticTreeError> {
        if self.phase == SessionPhase::Detached {
            return Err(SemanticTreeError::HostUnavailable);
        }
        if self.accessibility.is_some() {
            return Err(SemanticTreeError::ContainerAlreadyHasAccessibilityElements);
        }
        self.accessibility = Some(IosCustomRenderAccessibility::new(
            &self.render_host,
            self.marker,
        )?);
        Ok(())
    }

    /// Replace the active ordered accessibility snapshot.
    pub fn set_accessibility_nodes(
        &mut self,
        nodes: &[SemanticAccessibilityNode],
    ) -> core::result::Result<u32, SemanticTreeError> {
        if self.phase == SessionPhase::Detached {
            return Err(SemanticTreeError::HostUnavailable);
        }
        self.accessibility
            .as_mut()
            .ok_or(SemanticTreeError::HostUnavailable)?
            .set_nodes(nodes)
    }

    /// Remove the oldest generation-checked accessibility action request.
    pub fn take_accessibility_action(&mut self) -> Option<AccessibilityActionEvent> {
        self.accessibility
            .as_mut()
            .and_then(IosCustomRenderAccessibility::take_action)
    }

    /// Create one optional main-run-loop display callback for this session.
    pub fn enable_display_link<F>(&mut self, callback: F) -> Result<()>
    where
        F: FnMut(IosDisplayFrame) + 'static,
    {
        self.ensure_live()?;
        if self.display_link.is_some() {
            return Err(Error::new(ErrorKind::AlreadyExists));
        }
        let mut link = IosDisplayLink::new(self.marker, callback);
        if self.phase == SessionPhase::Suspended {
            link.pause();
        }
        self.display_link = Some(link);
        Ok(())
    }

    /// Apply scene foreground activity to the owned display and input adapters.
    pub fn set_active(&mut self, active: bool) -> Result<()> {
        self.ensure_live()?;
        let event = if active {
            SessionEvent::Activate
        } else {
            SessionEvent::Suspend
        };
        let (next, effect) = transition(self.phase, event);
        self.phase = next;
        match effect {
            SessionEffect::Activate => {
                if let Some(link) = &mut self.display_link {
                    link.resume();
                }
                if let Some(input) = &self.touch_input {
                    input.native_view().setUserInteractionEnabled(true);
                }
                if let Some(input) = &self.text_input {
                    input.native_text_view().setUserInteractionEnabled(true);
                }
            }
            SessionEffect::Suspend => {
                if let Some(link) = &mut self.display_link {
                    link.pause();
                }
                if let Some(input) = &self.touch_input {
                    input.cancel_active();
                    input.native_view().setUserInteractionEnabled(false);
                }
                if let Some(input) = &mut self.text_input {
                    let result = input.resign_focus();
                    input.native_text_view().setUserInteractionEnabled(false);
                    result?;
                }
            }
            SessionEffect::None | SessionEffect::Detach => {}
        }
        Ok(())
    }

    /// Return whether the session accepts frames and native input.
    pub fn is_active(&self) -> bool {
        self.phase == SessionPhase::Active
    }

    /// Permanently stop callbacks and remove every session-owned native child.
    ///
    /// The touch adapter remains retained only so callers may drain cancellation samples before
    /// dropping the session. Repeated calls are harmless; activation cannot reattach a detached
    /// session.
    pub fn detach(&mut self) {
        let (next, effect) = transition(self.phase, SessionEvent::Detach);
        self.phase = next;
        if effect != SessionEffect::Detach {
            return;
        }
        if let Some(link) = &mut self.display_link {
            link.invalidate();
        }
        self.text_input.take();
        self.accessibility.take();
        if let Some(input) = &mut self.touch_input {
            input.detach();
        }
        self.render_host.detach();
    }

    fn ensure_live(&self) -> Result<()> {
        if self.phase == SessionPhase::Detached {
            Err(Error::new(ErrorKind::Unavailable))
        } else {
            Ok(())
        }
    }
}

impl Drop for IosCustomRenderSession {
    fn drop(&mut self) {
        self.detach();
    }
}
