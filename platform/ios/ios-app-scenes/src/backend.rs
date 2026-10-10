use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use block2::RcBlock;
use ios_runtime::main_thread::MainThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSNotificationName, NSObjectProtocol, NSOperationQueue,
    NSRange, NSString,
};
use objc2_ui_kit::{
    UIScene, UISceneDidActivateNotification, UISceneDidDisconnectNotification,
    UISceneDidEnterBackgroundNotification, UISceneWillConnectNotification,
    UISceneWillDeactivateNotification, UISceneWillEnterForegroundNotification,
};

use crate::{
    EventQueue, NativeSceneNotification, SCENE_NOTIFICATIONS, SceneLifecycleEvent,
    SceneSessionIdentifier, contain_callback_panic, enqueue_if_live,
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
}

impl IosSceneLifecycleEvents {
    /// Register for the six public UIKit scene lifecycle notifications
    pub fn new(main_thread: MainThread) -> Self {
        let events = Arc::new(Mutex::new(EventQueue::new()));
        let mut observers = ObserverTokens::new(NSNotificationCenter::defaultCenter());
        let main_queue = NSOperationQueue::mainQueue();

        for notification in SCENE_NOTIFICATIONS {
            let name = notification_name(notification);
            let weak_events = Arc::downgrade(&events);
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
                        session_identifier: copy_identifier(&identifier),
                        kind,
                    };
                    enqueue_if_live(&weak_events, event);
                });
            });
            observers.register(name, &main_queue, &block);
        }

        Self {
            observers,
            _main_thread: main_thread,
            events,
        }
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
}

impl Drop for IosSceneLifecycleEvents {
    fn drop(&mut self) {
        self.observers.remove_all();
    }
}

fn copy_identifier(identifier: &NSString) -> SceneSessionIdentifier {
    let length = identifier.length();
    let mut code_units = vec![0_u16; length];
    if length != 0 {
        let buffer = NonNull::new(code_units.as_mut_ptr())
            .expect("a non-empty Vec has a non-null allocation pointer");
        // SAFETY: the buffer has `length` initialized slots and the range is the full immutable
        // NSString value measured immediately above.
        unsafe { identifier.getCharacters_range(buffer, NSRange::new(0, length)) };
    }
    SceneSessionIdentifier::from_owned_utf16_code_units(code_units)
}
