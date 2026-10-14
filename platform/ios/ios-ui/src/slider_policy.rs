use core::cell::{Cell, RefCell};
use std::collections::VecDeque;

use framework_core::{Error, ErrorKind, Result};

/// A finite, increasing closed range for a native `UISlider`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosSliderRange {
    minimum: f32,
    maximum: f32,
}

impl IosSliderRange {
    /// Creates a range with finite bounds and `minimum < maximum`.
    pub fn new(minimum: f32, maximum: f32) -> Result<Self> {
        if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns the inclusive lower bound.
    pub fn minimum(self) -> f32 {
        self.minimum
    }

    /// Returns the inclusive upper bound.
    pub fn maximum(self) -> f32 {
        self.maximum
    }

    /// Rejects non-finite values and clamps finite values to this range.
    pub fn clamp(self, value: f32) -> Result<f32> {
        if !value.is_finite() {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(value.clamp(self.minimum, self.maximum))
    }
}

/// Selects when UIKit reports slider changes through `UIControlEvents::ValueChanged`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosSliderReporting {
    /// Report value changes continuously while the user moves the thumb.
    Continuous,
    /// Report the value when the user finishes tracking the thumb.
    OnRelease,
}

impl IosSliderReporting {
    pub(crate) fn is_continuous(self) -> bool {
        matches!(self, Self::Continuous)
    }

    pub(crate) fn from_continuous(is_continuous: bool) -> Self {
        if is_continuous {
            Self::Continuous
        } else {
            Self::OnRelease
        }
    }
}

/// Validated initial value and reporting configuration for an `IosSlider`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosSliderConfig {
    range: IosSliderRange,
    initial_value: f32,
    reporting: IosSliderReporting,
}

impl IosSliderConfig {
    /// Creates a configuration, clamping a finite initial value to `range`.
    pub fn new(
        range: IosSliderRange,
        initial_value: f32,
        reporting: IosSliderReporting,
    ) -> Result<Self> {
        Ok(Self {
            range,
            initial_value: range.clamp(initial_value)?,
            reporting,
        })
    }

    /// Returns the configured range.
    pub fn range(self) -> IosSliderRange {
        self.range
    }

    /// Returns the clamped initial value.
    pub fn initial_value(self) -> f32 {
        self.initial_value
    }

    /// Returns the configured native reporting mode.
    pub fn reporting(self) -> IosSliderReporting {
        self.reporting
    }
}

pub(crate) struct SliderCallbackState {
    callback: RefCell<Box<dyn FnMut(f32)>>,
    pending: RefCell<VecDeque<f32>>,
    dispatching: Cell<bool>,
}

impl SliderCallbackState {
    pub(crate) fn new(callback: Box<dyn FnMut(f32)>) -> Self {
        Self {
            callback: RefCell::new(callback),
            pending: RefCell::new(VecDeque::new()),
            dispatching: Cell::new(false),
        }
    }
}

#[cfg(target_os = "ios")]
fn catch_slider_panic(callback: impl FnOnce()) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}

#[cfg(not(target_os = "ios"))]
fn catch_slider_panic(callback: impl FnOnce()) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback));
}

pub(crate) fn invoke_slider_callback(state: &SliderCallbackState, value: f32) {
    state.pending.borrow_mut().push_back(value);
    if state.dispatching.replace(true) {
        return;
    }
    loop {
        let next = state.pending.borrow_mut().pop_front();
        let Some(next) = next else {
            state.dispatching.set(false);
            return;
        };
        catch_slider_panic(|| {
            if let Ok(mut callback) = state.callback.try_borrow_mut() {
                callback(next);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IosSliderConfig, IosSliderRange, IosSliderReporting, SliderCallbackState,
        invoke_slider_callback,
    };
    use core::cell::RefCell;
    use std::rc::{Rc, Weak};

    #[test]
    fn range_rejects_non_finite_or_non_increasing_bounds() {
        assert!(IosSliderRange::new(f32::NAN, 1.0).is_err());
        assert!(IosSliderRange::new(0.0, f32::INFINITY).is_err());
        assert!(IosSliderRange::new(2.0, 2.0).is_err());
        assert!(IosSliderRange::new(3.0, 2.0).is_err());
    }

    #[test]
    fn config_and_updates_clamp_finite_values_and_reject_non_finite_values() {
        let range = IosSliderRange::new(-2.0, 8.0).unwrap();
        let config = IosSliderConfig::new(range, 10.0, IosSliderReporting::OnRelease).unwrap();
        assert_eq!(config.initial_value(), 8.0);
        assert_eq!(config.range().minimum(), -2.0);
        assert_eq!(config.range().maximum(), 8.0);
        assert_eq!(range.clamp(-10.0).unwrap(), -2.0);
        assert_eq!(range.clamp(4.0).unwrap(), 4.0);
        assert_eq!(config.reporting(), IosSliderReporting::OnRelease);
        assert!(IosSliderReporting::Continuous.is_continuous());
        assert_eq!(
            IosSliderReporting::from_continuous(false),
            IosSliderReporting::OnRelease
        );
        assert!(IosSliderConfig::new(range, f32::NAN, IosSliderReporting::Continuous).is_err());
        assert!(range.clamp(f32::NEG_INFINITY).is_err());
    }

    #[test]
    fn each_native_event_is_delivered_once_in_order_including_reentrant_events() {
        type CallbackSlot = Rc<RefCell<Option<Weak<SliderCallbackState>>>>;
        let callback_slot: CallbackSlot = Rc::new(RefCell::new(None));
        let seen = Rc::new(RefCell::new(Vec::new()));
        let nested_slot = callback_slot.clone();
        let nested_seen = seen.clone();
        let state = Rc::new(SliderCallbackState::new(Box::new(move |value| {
            nested_seen.borrow_mut().push(value);
            if value == 1.0 {
                let nested = callback_slot_for_upgrade(&nested_slot);
                invoke_slider_callback(&nested, 2.0);
                invoke_slider_callback(&nested, 3.0);
            }
        })));
        *callback_slot.borrow_mut() = Some(Rc::downgrade(&state));

        invoke_slider_callback(&state, 1.0);

        assert_eq!(&*seen.borrow(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn callback_panic_is_contained_and_later_events_are_delivered() {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let callback_seen = seen.clone();
        let state = SliderCallbackState::new(Box::new(move |value| {
            if value == 1.0 {
                panic!("slider callback panic");
            }
            callback_seen.borrow_mut().push(value);
        }));

        invoke_slider_callback(&state, 1.0);
        invoke_slider_callback(&state, 2.0);

        assert_eq!(&*seen.borrow(), &[2.0]);
        assert!(!state.dispatching.get());
    }

    fn callback_slot_for_upgrade(
        callback_slot: &Rc<RefCell<Option<Weak<SliderCallbackState>>>>,
    ) -> Rc<SliderCallbackState> {
        callback_slot
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .expect("callback target remains retained")
    }
}
