//! Main-thread ownership and copied frame timing for public QuartzCore display links.

use core::cell::RefCell;
use core::marker::PhantomData;
use std::rc::Rc;

use ios_runtime::ffi::catch_unwind;
use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send, sel};
use objc2_foundation::{NSObjectProtocol, NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::CADisplayLink;

use crate::display_link_policy::{CallbackGate, valid_frame_timing};

/// Copied timing values from one native display-link callback, in seconds.
///
/// These values describe the system's frame timing estimate, not completed rendering or a promise
/// that a callback occurs for every display refresh.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosDisplayFrame {
    timestamp_seconds: f64,
    target_timestamp_seconds: f64,
    duration_seconds: f64,
}

impl IosDisplayFrame {
    fn from_native(link: &CADisplayLink) -> Option<Self> {
        let timestamp_seconds = link.timestamp();
        let target_timestamp_seconds = link.targetTimestamp();
        let duration_seconds = link.duration();
        if !valid_frame_timing(
            timestamp_seconds,
            target_timestamp_seconds,
            duration_seconds,
        ) {
            return None;
        }
        Some(Self {
            timestamp_seconds,
            target_timestamp_seconds,
            duration_seconds,
        })
    }

    /// Return the timestamp of the most recent display update, in QuartzCore seconds.
    pub const fn timestamp_seconds(self) -> f64 {
        self.timestamp_seconds
    }

    /// Return the estimated timestamp of the next display update, in QuartzCore seconds.
    pub const fn target_timestamp_seconds(self) -> f64 {
        self.target_timestamp_seconds
    }

    /// Return the nominal duration of one display frame, in seconds.
    pub const fn duration_seconds(self) -> f64 {
        self.duration_seconds
    }
}

type DisplayFrameCallback = Box<dyn FnMut(IosDisplayFrame)>;

struct DisplayLinkTargetState {
    callback: RefCell<Option<DisplayFrameCallback>>,
    gate: CallbackGate,
}

define_class!(
    // SAFETY:
    // - The target is an NSObject subclass with no additional initializer requirements.
    // - The callback selector is registered only with CADisplayLink and is main-thread-only.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosDisplayLinkTarget"]
    #[ivars = DisplayLinkTargetState]
    struct DisplayLinkTarget;

    unsafe impl NSObjectProtocol for DisplayLinkTarget {}

    impl DisplayLinkTarget {
        #[unsafe(method(displayLinkDidFire:))]
        fn display_link_did_fire(&self, link: &CADisplayLink) {
            let _keep_alive = self.retain();
            let _ = catch_unwind(|| {
                if catch_unwind(|| self.deliver_frame(link)).is_err() {
                    self.ivars().gate.invalidate();
                    link.invalidate();
                    self.release_callback();
                }
            });
        }
    }
);

impl DisplayLinkTarget {
    fn new(marker: MainThreadMarker, callback: DisplayFrameCallback) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(DisplayLinkTargetState {
            callback: RefCell::new(Some(callback)),
            gate: CallbackGate::new(),
        });
        // SAFETY: this is a fresh NSObject subclass allocation and NSObject's initializer is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn deliver_frame(&self, link: &CADisplayLink) {
        let Some(_permit) = self.ivars().gate.begin_callback() else {
            return;
        };
        let Some(frame) = IosDisplayFrame::from_native(link) else {
            self.ivars().gate.invalidate();
            link.invalidate();
            self.release_callback();
            return;
        };
        if let Ok(mut callback) = self.ivars().callback.try_borrow_mut() {
            if let Some(callback) = callback.as_mut() {
                callback(frame);
            }
        }
        if !self.ivars().gate.is_active() {
            self.release_callback();
        }
    }

    fn release_callback(&self) {
        if let Ok(mut callback) = self.ivars().callback.try_borrow_mut() {
            callback.take();
        }
    }
}

/// A main-thread `CADisplayLink` owner with a Rust callback and no native timing-object escape.
///
/// Creation schedules the link on the main run loop in common modes and starts it immediately.
/// The callback receives finite, copied timing values; it runs synchronously on the main thread.
/// Reentrant ticks are skipped. A callback panic is contained and permanently invalidates the
/// link. `Drop` invalidates it. Display links follow run-loop and app lifecycle scheduling: they
/// do not promise background callbacks, fixed cadence, or runtime/vsync proof. Pause or invalidate
/// when the app's lifecycle policy requires it.
pub struct IosDisplayLink {
    native: Retained<CADisplayLink>,
    target: Retained<DisplayLinkTarget>,
    _not_send_or_sync: PhantomData<Rc<()>>,
}

impl IosDisplayLink {
    /// Create and start a display link on the caller-proven main thread.
    pub fn new(marker: MainThreadMarker, callback: impl FnMut(IosDisplayFrame) + 'static) -> Self {
        let target = DisplayLinkTarget::new(marker, Box::new(callback));
        // SAFETY: `target` implements the exact selector below and remains retained by this owner.
        let native = unsafe {
            CADisplayLink::displayLinkWithTarget_selector(&target, sel!(displayLinkDidFire:))
        };
        let run_loop = NSRunLoop::mainRunLoop();
        // SAFETY: both the owner and target are main-thread-only; the main run loop and common
        // mode constant are the documented CADisplayLink scheduling inputs.
        unsafe { native.addToRunLoop_forMode(&run_loop, NSRunLoopCommonModes) };
        Self {
            native,
            target,
            _not_send_or_sync: PhantomData,
        }
    }

    /// Pause callback delivery without invalidating the link.
    pub fn pause(&mut self) {
        self.target.ivars().gate.pause();
        if self.target.ivars().gate.is_active() {
            self.native.setPaused(true);
        }
    }

    /// Resume callback delivery after [`pause`](Self::pause), unless invalidated.
    pub fn resume(&mut self) {
        if self.target.ivars().gate.is_active() {
            self.target.ivars().gate.resume();
            self.native.setPaused(false);
        }
    }

    /// Permanently stop callbacks and release callback captures when no callback is in flight.
    pub fn invalidate(&mut self) {
        self.target.ivars().gate.invalidate();
        self.native.invalidate();
        self.target.release_callback();
    }

    /// Return whether the link remains valid, including after callback panic.
    pub fn is_valid(&self) -> bool {
        self.target.ivars().gate.is_active()
    }

    /// Return whether this owner has paused callback delivery.
    pub fn is_paused(&self) -> bool {
        self.target.ivars().gate.is_paused()
    }
}

impl Drop for IosDisplayLink {
    fn drop(&mut self) {
        self.invalidate();
    }
}
