use crate::mapping::application_state;
use crate::observers::{ObserverOwner, ObserverRemover, make_observer_callback};
use crate::queue::{EventQueue, LifecycleNotification};
use block2::RcBlock;
use framework_app::{ApplicationBackend, ApplicationState, LifecycleEvent};
use framework_core::{Availability, Result};
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_foundation::{
    NSDictionary, NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue, NSURL,
};
use objc2_ui_kit::{
    UIApplication, UIApplicationDidBecomeActiveNotification,
    UIApplicationDidEnterBackgroundNotification, UIApplicationOpenExternalURLOptionsKey,
    UIApplicationOpenSettingsURLString, UIApplicationWillEnterForegroundNotification,
    UIApplicationWillResignActiveNotification, UIApplicationWillTerminateNotification,
};
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, PoisonError};

type ObserverToken = Retained<ProtocolObject<dyn NSObjectProtocol>>;
type ObserverSet = ObserverOwner<Retained<NSNotificationCenter>, ObserverToken, 5>;
type ObserverBlock = dyn Fn(NonNull<NSNotification>);

/// Requests that UIKit open the current application's Settings page.
///
/// This fire-and-forget action uses UIKit's public `UIApplicationOpenSettingsURLString` and
/// `UIApplication.openURL:options:completionHandler:` APIs. It must run on the main thread. It
/// ignores UIKit's asynchronous open result and does not prove that Settings appeared or that the
/// user changed a setting. If Foundation cannot construct a URL from UIKit's fixed URL string, no
/// request is issued.
pub fn open_application_settings(main_thread: MainThread) {
    autoreleasepool(|_| {
        // SAFETY: UIKit exports this immutable NSString constant with process lifetime.
        let settings_url = unsafe { UIApplicationOpenSettingsURLString };
        let Some(url) = NSURL::URLWithString(settings_url) else {
            return;
        };
        let application = UIApplication::sharedApplication(main_thread.into_objc2());
        let options =
            NSDictionary::<UIApplicationOpenExternalURLOptionsKey, AnyObject>::dictionary();
        // SAFETY: `options` is an empty dictionary with the exact public UIKit key/value types;
        // no option values or completion block are supplied.
        unsafe {
            application.openURL_options_completionHandler(&url, &options, None);
        }
    });
}

/// iOS lifecycle state and nonblocking lifecycle events from public UIKit notifications.
///
/// Create and use this backend on UIKit's main thread. It owns its notification observer tokens
/// and removes them on drop. Its event queue holds 16 events; overflow discards the oldest event
/// to preserve the most recent state changes. Consecutive duplicate state transitions are
/// coalesced. After `UIApplicationWillTerminateNotification`, `state()` reports `Terminating` and
/// the event queue emits only `TerminationRequested`; UIKit provides no notification that proves
/// termination completed.
pub struct IosApplicationLifecycle {
    _main_thread: MainThreadMarker,
    application: Retained<UIApplication>,
    _observers: ObserverSet,
    events: Arc<Mutex<EventQueue>>,
}

impl IosApplicationLifecycle {
    /// Creates a lifecycle observer on the main thread without replacing app or scene delegates.
    pub fn new(main_thread: MainThread) -> Self {
        let marker = main_thread.into_objc2();
        let application = UIApplication::sharedApplication(marker);
        let initial_state = application_state(application.applicationState().0 as i32).ok();
        let events = Arc::new(Mutex::new(EventQueue::new(initial_state)));
        let center = NSNotificationCenter::defaultCenter();
        let queue = NSOperationQueue::mainQueue();
        // SAFETY: These generated UIKit extern statics are public, process-lifetime notification
        // names supplied by UIKit, as documented for the corresponding UIApplication events.
        let observers = unsafe {
            [
                add_observer(
                    &center,
                    &queue,
                    UIApplicationDidBecomeActiveNotification,
                    LifecycleNotification::DidBecomeActive,
                    &events,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillResignActiveNotification,
                    LifecycleNotification::WillResignActive,
                    &events,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillEnterForegroundNotification,
                    LifecycleNotification::WillEnterForeground,
                    &events,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationDidEnterBackgroundNotification,
                    LifecycleNotification::DidEnterBackground,
                    &events,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillTerminateNotification,
                    LifecycleNotification::WillTerminate,
                    &events,
                ),
            ]
        };
        Self {
            _main_thread: marker,
            application,
            _observers: ObserverOwner::new(center, observers),
            events,
        }
    }

    /// Returns UIKit's current state, or a platform error if UIKit reports an unknown state.
    pub fn state(&self) -> Result<ApplicationState> {
        if self.lock_events().is_terminating() {
            return Ok(ApplicationState::Terminating);
        }
        application_state(self.application.applicationState().0 as i32)
    }

    /// Returns one queued event without waiting for a future UIKit notification.
    pub fn poll_event(&mut self) -> Result<Option<LifecycleEvent>> {
        Ok(self.lock_events().pop())
    }

    fn lock_events(&self) -> std::sync::MutexGuard<'_, EventQueue> {
        self.events.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl ApplicationBackend for IosApplicationLifecycle {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn state(&self) -> Result<ApplicationState> {
        IosApplicationLifecycle::state(self)
    }

    fn poll_event(&mut self) -> Result<Option<LifecycleEvent>> {
        IosApplicationLifecycle::poll_event(self)
    }
}

impl ObserverRemover<ObserverToken> for Retained<NSNotificationCenter> {
    fn remove(&self, observer: &ObserverToken) {
        let object: &AnyObject = AsRef::<AnyObject>::as_ref(&**observer);
        // SAFETY: The token came from this notification center and has not been removed before.
        unsafe { self.removeObserver(object) };
    }
}

fn add_observer(
    center: &NSNotificationCenter,
    operation_queue: &NSOperationQueue,
    name: &'static objc2_foundation::NSNotificationName,
    notification: LifecycleNotification,
    events: &Arc<Mutex<EventQueue>>,
) -> ObserverToken {
    let callback = make_observer_callback(
        Arc::downgrade(events),
        move |events| events.apply(notification),
        contain_unwind,
    );
    let block: RcBlock<ObserverBlock> = RcBlock::new(move |_notification| callback());
    // SAFETY: UIKit lifecycle notifications use these static Foundation names and no object
    // filter. The main operation queue confines execution to UIKit's main thread. Captured
    // callback state uses `Weak<Mutex<_>>`, so the escaping block cannot keep detached event
    // state alive; its captures satisfy the API's sendability requirement. The center retains
    // the copied block until its observer token is removed.
    unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(name),
            None,
            Some(operation_queue),
            &block,
        )
    }
}

fn contain_unwind(callback: &mut (dyn FnMut() + '_)) {
    let _ = ios_runtime::ffi::catch_unwind(callback);
}
