//! Native UIKit touch delivery for a caller-owned custom-render host.

use core::cell::RefCell;
use std::ptr;

use ios_runtime::ffi::catch_unwind;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send};
use objc2_core_foundation::CGRect;
use objc2_foundation::{NSObjectProtocol, NSSet};
use objc2_ui_kit::{UIEvent, UIResponder, UITouch, UITouchType, UIView, UIViewAutoresizing};

use crate::custom_render_host::IosCustomRenderHost;
pub use crate::custom_render_touch_state::{
    IosTouchEvent, IosTouchInputFault, IosTouchKind, IosTouchPhase,
};
use crate::custom_render_touch_state::{MAX_ACTIVE_TOUCHES, TouchQueue, TouchSample};

struct TouchInputViewState {
    queue: RefCell<TouchQueue>,
}

define_class!(
    // SAFETY:
    // - UIView has no subclass initialization requirements beyond initWithFrame:.
    // - TouchInputView has no custom Drop implementation.
    #[unsafe(super(UIView, UIResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosCustomRenderTouchView"]
    #[ivars = TouchInputViewState]
    struct TouchInputView;

    impl TouchInputView {
        #[unsafe(method(didMoveToWindow))]
        fn did_move_to_window(&self) {
            let _ = catch_unwind(|| {
                if self.window().is_none() {
                    if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                        queue.cancel_all();
                    }
                }
            });
        }

        #[unsafe(method(touchesBegan:withEvent:))]
        fn touches_began(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| self.handle_began(touches, event));
        }

        #[unsafe(method(touchesMoved:withEvent:))]
        fn touches_moved(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| self.handle_moved(touches, event));
        }

        #[unsafe(method(touchesEnded:withEvent:))]
        fn touches_ended(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| self.handle_terminal(touches, IosTouchPhase::Ended, event));
        }

        #[unsafe(method(touchesCancelled:withEvent:))]
        fn touches_cancelled(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| {
                self.handle_terminal(touches, IosTouchPhase::Cancelled, event)
            });
        }
    }

    // SAFETY: NSObject supplies the default protocol methods; callbacks are main-thread-only.
    unsafe impl NSObjectProtocol for TouchInputView {}
);

impl TouchInputView {
    fn new(marker: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(TouchInputViewState {
            queue: RefCell::new(TouchQueue::new()),
        });
        // SAFETY: this is a fresh UIView subclass allocation and initWithFrame: is UIView's
        // documented initializer for a programmatically created custom view.
        let view: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };
        view.setOpaque(false);
        view.setBackgroundColor(None);
        view.setUserInteractionEnabled(true);
        view.setMultipleTouchEnabled(true);
        view.setAutoresizingMask(
            UIViewAutoresizing::FlexibleWidth | UIViewAutoresizing::FlexibleHeight,
        );
        view
    }

    fn handle_began(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
        let scale = self.host_scale();
        let button_mask = event_button_mask(event);
        let mut samples = [TouchSample::EMPTY; MAX_ACTIVE_TOUCHES];
        let mut count = 0;
        let mut excess_timestamp = None;
        for touch in touches.iter() {
            if count == MAX_ACTIVE_TOUCHES {
                excess_timestamp = Some(touch.timestamp());
                break;
            }
            samples[count] =
                sample_from_touch(&touch, native_key(&touch), self, scale, button_mask, false);
            count += 1;
        }
        if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
            queue.begin_batch(&mut samples[..count]);
            if let Some(timestamp) = excess_timestamp {
                queue.reject_contact_overflow(timestamp);
            }
        }
    }

    fn handle_moved(&self, touches: &NSSet<UITouch>, event: Option<&UIEvent>) {
        let mut ordered = [const { None }; MAX_ACTIVE_TOUCHES];
        let mut count = 0;
        let mut excess_timestamp = None;
        for touch in touches.iter() {
            if count == MAX_ACTIVE_TOUCHES {
                excess_timestamp = Some(touch.timestamp());
                break;
            }
            ordered[count] = Some(touch);
            count += 1;
        }
        if let Ok(queue) = self.ivars().queue.try_borrow() {
            ordered[..count].sort_by_key(|touch| {
                touch
                    .as_deref()
                    .and_then(|touch| queue.active_id(native_key(touch)))
                    .unwrap_or(u64::MAX)
            });
        }
        let scale = self.host_scale();
        let button_mask = event_button_mask(event);
        for touch in ordered[..count].iter().flatten() {
            let key = native_key(touch);
            let coalesced = event.and_then(|event| event.coalescedTouchesForTouch(touch));
            if let Some(samples) = coalesced.filter(|samples| !samples.is_empty()) {
                for sample in samples.iter() {
                    let sample = sample_from_touch(&sample, key, self, scale, button_mask, true);
                    if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                        queue.move_sample(sample);
                    }
                }
            } else {
                let sample = sample_from_touch(touch, key, self, scale, button_mask, false);
                if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                    queue.move_sample(sample);
                }
            }
        }
        if let Some(timestamp) = excess_timestamp {
            if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                queue.reject_contact_overflow(timestamp);
            }
        }
    }

    fn handle_terminal(
        &self,
        touches: &NSSet<UITouch>,
        phase: IosTouchPhase,
        event: Option<&UIEvent>,
    ) {
        let mut ordered = [const { None }; MAX_ACTIVE_TOUCHES];
        let mut count = 0;
        let mut excess_timestamp = None;
        for touch in touches.iter() {
            if count == MAX_ACTIVE_TOUCHES {
                excess_timestamp = Some(touch.timestamp());
                break;
            }
            ordered[count] = Some(touch);
            count += 1;
        }
        if let Ok(queue) = self.ivars().queue.try_borrow() {
            ordered[..count].sort_by_key(|touch| {
                touch
                    .as_deref()
                    .and_then(|touch| queue.active_id(native_key(touch)))
                    .unwrap_or(u64::MAX)
            });
        }
        let scale = self.host_scale();
        let button_mask = event_button_mask(event);
        for touch in ordered[..count].iter().flatten() {
            let sample =
                sample_from_touch(touch, native_key(touch), self, scale, button_mask, false);
            if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                queue.end(sample, phase);
            }
        }
        if let Some(timestamp) = excess_timestamp {
            if let Ok(mut queue) = self.ivars().queue.try_borrow_mut() {
                queue.reject_contact_overflow(timestamp);
            }
        }
    }

    fn host_scale(&self) -> f64 {
        self.superview().map_or_else(
            || self.contentScaleFactor(),
            |parent| parent.contentScaleFactor(),
        )
    }
}

fn native_key(touch: &UITouch) -> usize {
    ptr::from_ref(touch).cast::<()>() as usize
}

fn event_button_mask(event: Option<&UIEvent>) -> u64 {
    if objc2::available!(ios = 13.4, ..) {
        event.map_or(0, |event| event.buttonMask().0 as u64)
    } else {
        0
    }
}

fn sample_from_touch(
    touch: &UITouch,
    native_key: usize,
    view: &UIView,
    scale: f64,
    button_mask: u64,
    coalesced: bool,
) -> TouchSample {
    let point = touch.locationInView(Some(view));
    let kind = match touch.r#type() {
        UITouchType::Direct => IosTouchKind::Direct,
        UITouchType::Indirect => IosTouchKind::Indirect,
        UITouchType::Pencil => IosTouchKind::Pencil,
        UITouchType::IndirectPointer => IosTouchKind::IndirectPointer,
        other => IosTouchKind::Other(other.0 as i64),
    };
    TouchSample {
        native_key,
        timestamp_seconds: touch.timestamp(),
        x_points: point.x,
        y_points: point.y,
        scale,
        kind,
        tap_count: u32::try_from(touch.tapCount()).unwrap_or(u32::MAX),
        button_mask,
        coalesced,
    }
}

/// Main-thread UIKit adapter that queues raw multi-touch and pointer samples for a render host.
///
/// It adds one transparent UIView over the host. This view handles UIKit's documented responder
/// touch callbacks and therefore receives the contacts in its host bounds; a caller that adds
/// interactive native subviews above it may route those touches to the subviews instead. Gesture
/// recognition remains the caller's policy. The bounded event queue reserves room for cancellation
/// of every active contact. On overflow or invalid native coordinates it clears queued history,
/// emits cancellation for all delivered contacts, reports a sticky fault, and ignores later input
/// until the adapter is recreated.
pub struct IosCustomRenderTouchInput {
    view: Retained<TouchInputView>,
}

impl IosCustomRenderTouchInput {
    /// Attach an input overlay to a caller-owned custom-render host.
    pub fn attach(host: &IosCustomRenderHost, marker: MainThreadMarker) -> Self {
        let native = host.native_view();
        let view = TouchInputView::new(marker, native.bounds());
        native.addSubview(&view);
        Self { view }
    }

    /// Remove and permanently disable the overlay after queueing cancellations for active contacts.
    ///
    /// Call this before destruction when the application must drain terminal events afterward.
    pub fn detach(&mut self) {
        if let Ok(mut queue) = self.view.ivars().queue.try_borrow_mut() {
            queue.disable();
        }
        self.view.removeFromSuperview();
    }

    /// Remove the oldest queued event without allocation.
    pub fn next_event(&self) -> Option<IosTouchEvent> {
        self.view
            .ivars()
            .queue
            .try_borrow_mut()
            .ok()
            .and_then(|mut queue| queue.pop_event())
    }

    /// Queue cancellation for every active contact without disabling the overlay.
    ///
    /// Call this when the app or render host loses input focus without a UIKit cancellation callback.
    pub fn cancel_active(&self) {
        if let Ok(mut queue) = self.view.ivars().queue.try_borrow_mut() {
            queue.cancel_all();
        }
    }

    /// Return the sticky reason the adapter stopped event delivery, if any.
    pub fn fault(&self) -> Option<IosTouchInputFault> {
        self.view
            .ivars()
            .queue
            .try_borrow()
            .ok()
            .and_then(|queue| queue.fault())
    }

    /// Borrow the overlay's UIView for public UIKit customization.
    pub fn native_view(&self) -> &UIView {
        &self.view
    }
}

impl Drop for IosCustomRenderTouchInput {
    fn drop(&mut self) {
        if let Ok(mut queue) = self.view.ivars().queue.try_borrow_mut() {
            queue.disable();
        }
        self.view.removeFromSuperview();
    }
}
