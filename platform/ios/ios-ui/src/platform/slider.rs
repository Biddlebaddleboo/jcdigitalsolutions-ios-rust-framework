use core::marker::PhantomData;
use std::rc::Rc;

use framework_core::Result;
use framework_ui::Frame;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_foundation::NSObjectProtocol;
use objc2_ui_kit::{UIControlEvents, UISlider};

use crate::slider_policy::{
    IosSliderConfig, IosSliderRange, IosSliderReporting, SliderCallbackState,
    invoke_slider_callback,
};

use super::{IosUiBackend, IosView, main_thread_marker, native_frame};

/// A retained UIKit slider and its `ValueChanged` callback owner.
///
/// Keep this handle alive while its callback should remain active. Dropping it unregisters the
/// exact target/action even when the parent still retains the native slider view.
pub struct IosSlider {
    native: Retained<UISlider>,
    _target: Retained<SliderTarget>,
    _not_send: PhantomData<Rc<()>>,
}

struct SliderTargetState {
    callback: SliderCallbackState,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - SliderTarget does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosUiSliderTarget"]
    #[ivars = SliderTargetState]
    struct SliderTarget;

    impl SliderTarget {
        #[unsafe(method(sliderValueChanged:))]
        fn slider_value_changed(&self, sender: &UISlider) {
            let _keep_alive = self.retain();
            invoke_slider_callback(&self.ivars().callback, sender.value());
        }
    }

    // SAFETY: NSObjectProtocol adds no required methods, and NSObject supplies its implementation.
    unsafe impl NSObjectProtocol for SliderTarget {}
);

impl SliderTarget {
    fn with_action(mtm: MainThreadMarker, action: Box<dyn FnMut(f32)>) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(SliderTargetState {
            callback: SliderCallbackState::new(action),
        });
        // SAFETY: `this` is a fresh NSObject subclass allocation and NSObject's `init` is valid.
        unsafe { msg_send![super(this), init] }
    }
}

impl IosUiBackend {
    /// Adds a native `UISlider` configured with a finite range, initial value, and reporting mode.
    ///
    /// Finite initial values outside the closed range are clamped by `config`. The callback is
    /// invoked once for each `UIControlEvents::ValueChanged` event delivered to this target;
    /// programmatic value updates are silent. The caller owns layout and hierarchy placement.
    pub fn add_slider<F>(
        &mut self,
        parent: &mut IosView,
        frame: Frame,
        config: IosSliderConfig,
        action: F,
    ) -> Result<IosSlider>
    where
        F: FnMut(f32) + 'static,
    {
        let marker = main_thread_marker()?;
        let native = UISlider::new(marker);
        native.setFrame(native_frame(frame));
        native.setMaximumValue(config.range().maximum());
        native.setMinimumValue(config.range().minimum());
        native.setContinuous(config.reporting().is_continuous());
        native.setValue_animated(config.initial_value(), false);
        let target = SliderTarget::with_action(marker, Box::new(action));
        // SAFETY: `target` is strongly retained by IosSlider and implements the exact
        // `sliderValueChanged:` selector registered for UIControlEvents::ValueChanged.
        unsafe {
            native.addTarget_action_forControlEvents(
                Some(&*target),
                sel!(sliderValueChanged:),
                UIControlEvents::ValueChanged,
            );
        }
        parent.native.addSubview(&native);
        Ok(IosSlider {
            native,
            _target: target,
            _not_send: PhantomData,
        })
    }
}

impl IosSlider {
    /// Reads the slider's current value on the UIKit main thread.
    pub fn value(&self) -> Result<f32> {
        let _slider_value_thread = main_thread_marker()?;
        Ok(self.native.value())
    }

    /// Sets a finite value, clamps it to the current native range, and returns the applied value.
    ///
    /// This programmatic update does not synthesize `ValueChanged` or invoke the callback.
    pub fn set_value(&mut self, value: f32, animated: bool) -> Result<f32> {
        let _slider_value_thread = main_thread_marker()?;
        let range = IosSliderRange::new(self.native.minimumValue(), self.native.maximumValue())?;
        let value = range.clamp(value)?;
        self.native.setValue_animated(value, animated);
        Ok(value)
    }

    /// Reads the current native continuous-versus-release reporting mode.
    pub fn reporting(&self) -> Result<IosSliderReporting> {
        let _slider_reporting_thread = main_thread_marker()?;
        Ok(IosSliderReporting::from_continuous(
            self.native.isContinuous(),
        ))
    }

    /// Selects continuous thumb tracking or a single release report through UIKit.
    pub fn set_reporting(&mut self, reporting: IosSliderReporting) -> Result<()> {
        let _slider_reporting_thread = main_thread_marker()?;
        self.native.setContinuous(reporting.is_continuous());
        Ok(())
    }

    /// Borrows the native slider for caller-owned UIKit integration.
    pub fn native_slider(&self) -> &UISlider {
        &self.native
    }
}

impl Drop for IosSlider {
    fn drop(&mut self) {
        // SAFETY: the handle is main-thread-only and removes only its retained action for the
        // exact selector and ValueChanged event registered above.
        unsafe {
            self.native.removeTarget_action_forControlEvents(
                Some(&*self._target),
                Some(sel!(sliderValueChanged:)),
                UIControlEvents::ValueChanged,
            );
        }
    }
}
