use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use block2::RcBlock;
use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSNotificationName, NSObjectProtocol, NSOperationQueue,
};
use objc2_ui_kit::{
    UIScene, UISceneDidActivateNotification, UISceneDidDisconnectNotification,
    UISceneDidEnterBackgroundNotification, UISceneWillConnectNotification,
    UISceneWillDeactivateNotification, UISceneWillEnterForegroundNotification,
};

use crate::{
    EventQueue, NativeSceneNotification, SCENE_NOTIFICATIONS, SceneEventHandlerState,
    SceneLifecycleEvent, contain_callback_panic, copy_session_identifier, publish_event_if_live,
};

fn notification_name(notification: NativeSceneNotification) -> &'static NSNotificationName {
    // SAFETY: these are immutable public UIKit notification names with static storage.
    unsafe {
        match notification {
            NativeSceneNotification::WillConnect => UISceneWillConnectNotification,
            NativeSceneNotification::DidDisconnect => UISceneDidDisconnectNotification,
            NativeSceneNotification::DidActivate => UISceneDidActivateNotification,
            NativeSceneNotification::WillDeactivate => UISceneWillDeactivateNotification,
            NativeSceneNotification::WillEnterForeground => UISceneWillEnterForegroundNotification,
            NativeSceneNotification::DidEnterBackground => UISceneDidEnterBackgroundNotification,
        }
    }
}

type ObserverToken = Retained<ProtocolObject<dyn NSObjectProtocol>>;

struct ObserverTokens {
    center: Retained<NSNotificationCenter>,
    tokens: Vec<ObserverToken>,
}

impl ObserverTokens {
    fn new(center: Retained<NSNotificationCenter>) -> Self {
        Self {
            center,
            tokens: Vec::with_capacity(6),
        }
    }

    fn register(
        &mut self,
        name: &NSNotificationName,
        queue: &NSOperationQueue,
        block: &block2::DynBlock<dyn Fn(NonNull<NSNotification>)>,
    ) {
        // SAFETY: the block captures only Send + Sync values and is routed to the main queue;
        // the notification name is one of UIKit's public scene lifecycle names.
        let token = unsafe {
            self.center.addObserverForName_object_queue_usingBlock(
                Some(name),
                None,
                Some(queue),
                block,
            )
        };
        self.tokens.push(token);
    }

    fn remove_all(&mut self) {
        for token in self.tokens.drain(..) {
            // SAFETY: this token is the NSObjectProtocol returned by this center's registration.
            let token_ref: &ProtocolObject<dyn NSObjectProtocol> = token.as_ref();
            let token_object: &AnyObject = token_ref.as_ref();
            unsafe { self.center.removeObserver(token_object) };
        }
    }
}

impl Drop for ObserverTokens {
    fn drop(&mut self) {
        self.remove_all();
    }
}

/// Receives UIKit scene lifecycle events without replacing the host's scene delegate
///
/// Construct and use this event source on UIKit's main thread. It observes only after
/// construction, queues no initial event, and never creates or controls a scene. Events identify
/// their source with the exact UTF-16 code units of `UISceneSession.persistentIdentifier`.
pub struct IosSceneLifecycleEvents {
    observers: ObserverTokens,
    _main_thread: MainThread,
    events: Arc<Mutex<EventQueue>>,
    handler: Arc<SceneEventHandlerState>,
}

impl IosSceneLifecycleEvents {
    /// Register for the six public UIKit scene lifecycle notifications
    pub fn new(main_thread: MainThread) -> Self {
        let events = Arc::new(Mutex::new(EventQueue::new()));
        let handler = Arc::new(SceneEventHandlerState::new());
        let mut observers = ObserverTokens::new(NSNotificationCenter::defaultCenter());
        let main_queue = NSOperationQueue::mainQueue();

        for notification in SCENE_NOTIFICATIONS {
            let name = notification_name(notification);
            let weak_events = Arc::downgrade(&events);
            let weak_handler = Arc::downgrade(&handler);
            let kind = notification.event_kind();
            let block = RcBlock::new(move |notification: NonNull<NSNotification>| {
                let _ = contain_callback_panic(|| {
                    // SAFETY: Notification Center supplies a live notification object for the
                    // duration of this block invocation.
                    let notification = unsafe { notification.as_ref() };
                    let Some(object) = notification.object() else {
                        return;
                    };
                    let Ok(scene) = object.downcast::<UIScene>() else {
                        return;
                    };
                    let identifier = scene.session().persistentIdentifier();
                    let event = SceneLifecycleEvent {
                        session_identifier: copy_session_identifier(&identifier),
                        kind,
                    };
                    publish_event_if_live(&weak_events, &weak_handler, event);
                });
            });
            observers.register(name, &main_queue, &block);
        }

        Self {
            observers,
            _main_thread: main_thread,
            events,
            handler,
        }
    }

    /// Install or replace a direct callback for subsequent scene lifecycle events
    ///
    /// The callback runs on `NSOperationQueue::mainQueue()` after the event has been added to
    /// the polling queue. It receives events independently of polling-queue overflow. The
    /// callback must be `Send` so its synchronized `Arc<Mutex<...>>` capture satisfies the
    /// sendable-block safety contract of `NSNotificationCenter::addObserverForName:object:queue:usingBlock:`;
    /// callback execution remains on UIKit's main operation queue. Panics are contained.
    /// If a callback causes nested scene notifications, those events are delivered after it
    /// returns rather than recursively invoking this `FnMut`. The nested-delivery backlog holds
    /// at most 32 events; excess nested events drop oldest and increment
    /// `dropped_handler_event_count()`.
    pub fn set_event_handler(&mut self, handler: impl FnMut(SceneLifecycleEvent) + Send + 'static) {
        self.handler.set(handler);
    }

    /// Clear the direct callback without changing polling-queue contents
    pub fn clear_event_handler(&mut self) {
        self.handler.clear();
    }

    /// Remove and return the oldest queued scene event without waiting
    pub fn poll_event(&mut self) -> Option<SceneLifecycleEvent> {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop()
    }

    /// Return the cumulative number of events dropped because the bounded queue was full
    pub fn dropped_event_count(&self) -> u64 {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .dropped()
    }

    /// Return the cumulative number of nested callback events dropped at the callback backlog limit
    pub fn dropped_handler_event_count(&self) -> u64 {
        self.handler.dropped()
    }
}

impl Drop for IosSceneLifecycleEvents {
    fn drop(&mut self) {
        self.observers.remove_all();
    }
}
