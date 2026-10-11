use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use framework_ui::Frame;
use objc2::rc::Retained;
use objc2_ui_kit::{UIActivityIndicatorView, UIActivityIndicatorViewStyle};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

/// A retained UIKit activity indicator with main-thread affinity.
pub struct IosActivityIndicator {
    native: Retained<UIActivityIndicatorView>,
    _not_send: PhantomData<Rc<()>>,
}

impl IosUiBackend {
    /// Adds a medium UIKit activity indicator to `parent` with the requested initial state.
    ///
    /// The indicator uses `hidesWhenStopped(true)`: `start` shows it, and `stop` hides it. The
    /// caller owns the frame and hierarchy placement.
    pub fn add_activity_indicator(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        animating: bool,
    ) -> Result<IosActivityIndicator> {
        let marker = main_thread_marker()?;
        let native = UIActivityIndicatorView::new(marker);
        native.setActivityIndicatorViewStyle(UIActivityIndicatorViewStyle::Medium);
        native.setHidesWhenStopped(true);
        native.setFrame(native_frame(frame));
        if animating {
            native.startAnimating();
        } else {
            native.stopAnimating();
        }
        parent.native.addSubview(&native);
        Ok(IosActivityIndicator {
            native,
            _not_send: PhantomData,
        })
    }
}

impl IosActivityIndicator {
    /// Starts animation and makes the indicator visible unless an ancestor view is hidden.
    pub fn start(&mut self) -> Result<()> {
        let _activity_indicator_thread = main_thread_marker()?;
        self.native.startAnimating();
        Ok(())
    }

    /// Stops animation; UIKit hides the indicator because `hidesWhenStopped` is enabled.
    pub fn stop(&mut self) -> Result<()> {
        let _activity_indicator_thread = main_thread_marker()?;
        self.native.stopAnimating();
        Ok(())
    }

    /// Returns UIKit's current animation state.
    pub fn is_animating(&self) -> Result<bool> {
        let _activity_indicator_thread = main_thread_marker()?;
        Ok(self.native.isAnimating())
    }

    /// Borrows the native activity indicator for UIKit integration.
    pub fn native_activity_indicator(&self) -> &UIActivityIndicatorView {
        &self.native
    }
}

impl Drop for IosActivityIndicator {
    fn drop(&mut self) {
        // This handle is !Send and its creation requires a main-thread proof.
        self.native.stopAnimating();
    }
}
