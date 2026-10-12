use crate::mapping::application_state;
use crate::observers::{
    LifecycleEventHandlerState, ObserverOwner, ObserverRemover, make_observer_callback,
};
use crate::queue::{EventQueue, LifecycleNotification};
use block2::RcBlock;
use core::cell::RefCell;
use framework_app::{ApplicationBackend, ApplicationState, LifecycleEvent};
use framework_core::{Availability, Result};
use ios_runtime::main_thread::MainThread;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject, Bool, ProtocolObject};
use objc2_foundation::{
    NSDictionary, NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue,
    NSString, NSURL,
};
use objc2_ui_kit::{
    UIApplication, UIApplicationDidBecomeActiveNotification,
    UIApplicationDidEnterBackgroundNotification, UIApplicationOpenExternalURLOptionsKey,
    UIApplicationOpenSettingsURLString, UIApplicationWillEnterForegroundNotification,
    UIApplicationWillResignActiveNotification, UIApplicationWillTerminateNotification,
};
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

type ObserverToken = Retained<ProtocolObject<dyn NSObjectProtocol>>;
type ObserverSet = ObserverOwner<Retained<NSNotificationCenter>, ObserverToken, 5>;
type ObserverBlock = dyn Fn(NonNull<NSNotification>);

/// A bounded reason UIKit cannot receive a caller-supplied application URI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenApplicationUriError {
    /// Foundation rejected the caller's URI text.
    FoundationRejectedUri,
    /// The constructed URL has no absolute URI scheme.
    MissingScheme,
}

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

/// Requests that UIKit open an absolute URI with its system handler.
///
/// `uri` is passed unchanged to Foundation. The constructed URL must have a scheme, but no host,
/// HTTPS-only rule, or scheme allowlist is imposed. On success, `completion` receives UIKit's
/// Boolean open result at most once; a `true` result does not prove that another app launched,
/// content loaded, or a user saw it. An error means that Foundation rejected the URI or it had no
/// scheme; in either case no UIKit request is issued and the completion is not called. Foreground
/// checks and any application-specific scheme policy remain the caller's responsibility. This API
/// uses `UIApplication` and is not available to app extensions.
pub fn open_application_uri(
    main_thread: MainThread,
    uri: &str,
    completion: impl FnOnce(bool) + 'static,
) -> core::result::Result<(), OpenApplicationUriError> {
    autoreleasepool(|_| {
        let text = NSString::from_str(uri);
        let Some(url) = NSURL::URLWithString(&text) else {
            return Err(OpenApplicationUriError::FoundationRejectedUri);
        };
        if url.scheme().is_none_or(|scheme| scheme.length() == 0) {
            return Err(OpenApplicationUriError::MissingScheme);
        }

        let callback = Rc::new(RefCell::new(Some(completion)));
        let handler = RcBlock::new(move |opened: Bool| {
            let _ = ios_runtime::ffi::catch_unwind(|| {
                let callback = callback.borrow_mut().take();
                if let Some(callback) = callback {
                    callback(opened.as_bool());
                }
            });
        });
        let application = UIApplication::sharedApplication(main_thread.into_objc2());
        let options =
            NSDictionary::<UIApplicationOpenExternalURLOptionsKey, AnyObject>::dictionary();
        // SAFETY: `options` uses the public UIKit key/value types and is empty. UIKit copies and
        // invokes the completion block for the asynchronous URL-open result.
        unsafe {
            application.openURL_options_completionHandler(&url, &options, Some(&handler));
        }
        Ok(())
    })
}

/// iOS lifecycle state and nonblocking lifecycle events from public UIKit notifications.
///
/// Create and use this backend on UIKit's main thread. It owns its notification observer tokens
/// and removes them on drop. Its event queue holds 16 events; overflow discards the oldest event
/// to preserve the most recent state changes. Consecutive duplicate state transitions are
/// coalesced. After `UIApplicationWillTerminateNotification`, `state()` reports `Terminating` and
/// the event queue emits only `TerminationRequested`; UIKit provides no notification that proves
/// termination completed. An optional direct event handler runs synchronously on the main
/// operation queue after each accepted event is also added to the polling queue. Its independent
/// reentrant backlog holds 16 events and drops oldest on overflow; use
/// `dropped_handler_event_count()` to read that overflow count.
pub struct IosApplicationLifecycle {
    _main_thread: MainThreadMarker,
    application: Retained<UIApplication>,
    _observers: ObserverSet,
    events: Arc<Mutex<EventQueue>>,
    handler: Arc<LifecycleEventHandlerState>,
}

impl IosApplicationLifecycle {
    /// Creates a lifecycle observer on the main thread without replacing app or scene delegates.
    pub fn new(main_thread: MainThread) -> Self {
        let marker = main_thread.into_objc2();
        let application = UIApplication::sharedApplication(marker);
        let initial_state = application_state(application.applicationState().0 as i32).ok();
        let events = Arc::new(Mutex::new(EventQueue::new(initial_state)));
        let handler = Arc::new(LifecycleEventHandlerState::new());
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
                    &handler,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillResignActiveNotification,
                    LifecycleNotification::WillResignActive,
                    &events,
                    &handler,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillEnterForegroundNotification,
                    LifecycleNotification::WillEnterForeground,
                    &events,
                    &handler,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationDidEnterBackgroundNotification,
                    LifecycleNotification::DidEnterBackground,
                    &events,
                    &handler,
                ),
                add_observer(
                    &center,
                    &queue,
                    UIApplicationWillTerminateNotification,
                    LifecycleNotification::WillTerminate,
                    &events,
                    &handler,
                ),
            ]
        };
        Self {
            _main_thread: marker,
            application,
            _observers: ObserverOwner::new(center, observers),
            events,
            handler,
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

    /// Installs or replaces a direct callback for subsequent lifecycle events.
    ///
    /// The callback runs synchronously on `NSOperationQueue::mainQueue()` after the event is
    /// added to the polling queue, and runs independently of polling-queue overflow. It must be
    /// `Send` so the observer block satisfies Notification Center's sendability contract; UIKit
    /// still invokes it on the main operation queue. A callback panic is contained. Nested
    /// notifications are delivered after the current `FnMut` returns, through a 16-event
    /// drop-oldest backlog. Replacing the handler discards pending events for the old handler.
    /// Clearing or dropping the backend rejects pending and later stale delivery, but cannot
    /// interrupt a callback already executing.
    pub fn set_event_handler(&mut self, handler: impl FnMut(LifecycleEvent) + Send + 'static) {
        self.handler.set(handler);
    }

    /// Clears the direct callback without changing polling-queue contents.
    pub fn clear_event_handler(&mut self) {
        self.handler.clear();
    }

    /// Returns the cumulative, saturating count of nested callback events dropped at the 16-event limit.
    pub fn dropped_handler_event_count(&self) -> u64 {
        self.handler.dropped()
    }

    fn lock_events(&self) -> std::sync::MutexGuard<'_, EventQueue> {
        self.events.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for IosApplicationLifecycle {
    fn drop(&mut self) {
        self.handler.clear();
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
    handler: &Arc<LifecycleEventHandlerState>,
) -> ObserverToken {
    let callback = make_observer_callback(
        Arc::downgrade(events),
        Arc::downgrade(handler),
        notification,
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
