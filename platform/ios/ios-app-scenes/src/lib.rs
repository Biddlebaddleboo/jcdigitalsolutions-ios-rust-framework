#![deny(missing_docs)]
#![doc = "UIKit scene activity snapshots and lifecycle events for iOS apps"]

#[cfg(any(target_os = "ios", test))]
use framework_core::{Error, ErrorKind, Result};
#[cfg(any(target_os = "ios", test))]
use std::collections::VecDeque;
#[cfg(test)]
use std::sync::Arc;
#[cfg(any(target_os = "ios", test))]
use std::sync::{Mutex, Weak};

#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use objc2_foundation::{NSRange, NSString};
#[cfg(target_os = "ios")]
use objc2_ui_kit::{
    UIApplication, UISceneActivationState as NativeSceneActivationState, UIWindowScene,
};
#[cfg(target_os = "ios")]
use std::ptr::NonNull;

#[cfg(target_os = "ios")]
mod backend;

#[cfg(target_os = "ios")]
mod application_host;

#[cfg(target_os = "ios")]
mod scene_host;

#[cfg(any(target_os = "ios", test))]
mod callback_queue;

#[cfg(any(target_os = "ios", test))]
mod scene_phase;

#[cfg(target_os = "ios")]
pub use backend::IosSceneLifecycleEvents;

#[cfg(target_os = "ios")]
mod window;

#[cfg(target_os = "ios")]
pub use window::IosSceneWindow;

#[cfg(target_os = "ios")]
pub use application_host::{IosApplicationHandler, IosApplicationHost};

#[cfg(target_os = "ios")]
pub use scene_host::{IosSceneHost, IosScenePhase};

/// Counts scene activation states from one `UIApplication.connectedScenes` read
#[cfg(target_os = "ios")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneActivitySnapshot {
    connected_scene_count: usize,
    unattached_count: usize,
    foreground_active_count: usize,
    foreground_inactive_count: usize,
    background_count: usize,
    unknown_state_count: usize,
}

#[cfg(target_os = "ios")]
impl SceneActivitySnapshot {
    const fn empty() -> Self {
        Self {
            connected_scene_count: 0,
            unattached_count: 0,
            foreground_active_count: 0,
            foreground_inactive_count: 0,
            background_count: 0,
            unknown_state_count: 0,
        }
    }

    /// Count of scene values returned by `UIApplication.connectedScenes`
    pub const fn connected_scene_count(self) -> usize {
        self.connected_scene_count
    }

    /// Count of scenes with `UISceneActivationState::Unattached`
    pub const fn unattached_count(self) -> usize {
        self.unattached_count
    }

    /// Count of scenes with `UISceneActivationState::ForegroundActive`
    pub const fn foreground_active_count(self) -> usize {
        self.foreground_active_count
    }

    /// Count of scenes with `UISceneActivationState::ForegroundInactive`
    pub const fn foreground_inactive_count(self) -> usize {
        self.foreground_inactive_count
    }

    /// Count of scenes with `UISceneActivationState::Background`
    pub const fn background_count(self) -> usize {
        self.background_count
    }

    /// Count of scene states unknown to this binding version
    pub const fn unknown_state_count(self) -> usize {
        self.unknown_state_count
    }

    fn record(&mut self, state: NativeSceneActivationState) {
        self.connected_scene_count += 1;
        match state {
            NativeSceneActivationState::Unattached => self.unattached_count += 1,
            NativeSceneActivationState::ForegroundActive => self.foreground_active_count += 1,
            NativeSceneActivationState::ForegroundInactive => self.foreground_inactive_count += 1,
            NativeSceneActivationState::Background => self.background_count += 1,
            _ => self.unknown_state_count += 1,
        }
    }
}

/// Read the app's scene activation-state counts on UIKit's main thread
///
/// This reads `UIApplication.connectedScenes` once and each returned `UIScene.activationState`
/// once in one synchronous main-thread pass. It does not promise an atomic cross-scene instant,
/// subscribe to changes, retain a scene, or report whether any scene is onscreen
#[cfg(target_os = "ios")]
pub fn snapshot(main_thread: MainThread) -> SceneActivitySnapshot {
    let application = UIApplication::sharedApplication(main_thread.into_objc2());
    let scenes = application.connectedScenes().allObjects();
    let mut snapshot = SceneActivitySnapshot::empty();
    for scene in scenes.iter() {
        snapshot.record(scene.activationState());
    }
    snapshot
}

/// One connected window scene's activation state and exact session identity
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneActivationSnapshot {
    session_identifier: SceneSessionIdentifier,
    state: SceneActivationState,
}

#[cfg(any(target_os = "ios", test))]
impl SceneActivationSnapshot {
    /// Return the persistent identity for this scene's UIKit session
    pub const fn session_identifier(&self) -> &SceneSessionIdentifier {
        &self.session_identifier
    }

    /// Return the activation state sampled for this scene
    pub const fn state(&self) -> SceneActivationState {
        self.state
    }
}

/// UIKit activation state for one connected window scene
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneActivationState {
    /// UIKit reports that the scene is not attached to the app
    Unattached,
    /// The scene is in the foreground and active
    ForegroundActive,
    /// The scene is in the foreground but inactive
    ForegroundInactive,
    /// The scene is in the background
    Background,
    /// UIKit returned an activation value unknown to this binding version
    Unknown(i64),
}

#[cfg(any(target_os = "ios", test))]
impl SceneActivationState {
    fn from_native_value(value: i64) -> Self {
        match value {
            -1 => Self::Unattached,
            0 => Self::ForegroundActive,
            1 => Self::ForegroundInactive,
            2 => Self::Background,
            value => Self::Unknown(value),
        }
    }
}

/// Read current connected `UIWindowScene` activation states on UIKit's main thread
///
/// The result owns each session identifier as exact UTF-16 code units and retains no UIKit
/// scene or window. The read is a synchronous snapshot, not an atomic cross-scene instant or a
/// guarantee about state after return. Scene order is unspecified; non-window `UIScene` values
/// are omitted. Use `IosSceneLifecycleEvents` for subsequent transitions
#[cfg(target_os = "ios")]
pub fn scene_activation_snapshot(main_thread: MainThread) -> Vec<SceneActivationSnapshot> {
    let application = UIApplication::sharedApplication(main_thread.into_objc2());
    let scenes = application.connectedScenes().allObjects();
    let mut snapshots = Vec::with_capacity(scenes.len());
    for scene in scenes.iter() {
        let Some(window_scene) = scene.downcast_ref::<UIWindowScene>() else {
            continue;
        };
        let session = window_scene.session();
        let identifier = session.persistentIdentifier();
        let state = window_scene.activationState();
        snapshots.push(SceneActivationSnapshot {
            session_identifier: copy_session_identifier(&identifier),
            state: SceneActivationState::from_native_value(state.0 as i64),
        });
    }
    snapshots
}

#[cfg(target_os = "ios")]
pub(crate) fn copy_session_identifier(identifier: &NSString) -> SceneSessionIdentifier {
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

#[cfg(any(target_os = "ios", test))]
const EVENT_CAPACITY: usize = 32;
#[cfg(any(target_os = "ios", test))]
const HANDLER_PENDING_CAPACITY: usize = 32;

/// A lossless copy of a UIKit scene session's persistent identifier
///
/// The code units are private. `to_string_lossless` returns `None` only if the native identifier
/// contains an unpaired UTF-16 surrogate; `as_utf16_code_units` always preserves the full value
/// for that case and for identifiers containing embedded NUL characters
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SceneSessionIdentifier(Box<[u16]>);

#[cfg(any(target_os = "ios", test))]
impl SceneSessionIdentifier {
    /// Copy an identifier from UTF-16 code units without normalization or replacement
    pub fn from_utf16_code_units(code_units: &[u16]) -> Self {
        Self(code_units.into())
    }

    #[cfg(target_os = "ios")]
    pub(crate) fn from_owned_utf16_code_units(code_units: Vec<u16>) -> Self {
        Self(code_units.into_boxed_slice())
    }

    /// Return the exact UTF-16 code units copied from UIKit
    pub fn as_utf16_code_units(&self) -> &[u16] {
        &self.0
    }

    /// Convert to a Rust string when the identifier contains valid UTF-16
    pub fn to_string_lossless(&self) -> Option<String> {
        String::from_utf16(&self.0).ok()
    }
}

/// A lifecycle transition reported by a UIKit scene notification
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneLifecycleEventKind {
    /// UIKit is about to connect the scene to the app
    WillConnect,
    /// UIKit disconnected the scene from the app
    DidDisconnect,
    /// UIKit activated the scene
    DidActivate,
    /// UIKit is about to deactivate the scene
    WillDeactivate,
    /// UIKit is about to move the scene into the foreground
    WillEnterForeground,
    /// UIKit moved the scene into the background
    DidEnterBackground,
}

/// One per-scene lifecycle transition and the session identity UIKit attached to it
#[cfg(any(target_os = "ios", test))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneLifecycleEvent {
    session_identifier: SceneSessionIdentifier,
    kind: SceneLifecycleEventKind,
}

#[cfg(any(target_os = "ios", test))]
impl SceneLifecycleEvent {
    /// Return the UIKit session identity for the scene that emitted this event
    pub const fn session_identifier(&self) -> &SceneSessionIdentifier {
        &self.session_identifier
    }

    /// Return the scene lifecycle transition
    pub const fn kind(&self) -> SceneLifecycleEventKind {
        self.kind
    }
}

#[cfg(any(target_os = "ios", test))]
struct EventQueue {
    events: VecDeque<SceneLifecycleEvent>,
    dropped: u64,
}

#[cfg(any(target_os = "ios", test))]
impl EventQueue {
    fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(EVENT_CAPACITY),
            dropped: 0,
        }
    }

    fn push(&mut self, event: SceneLifecycleEvent) {
        if self.events.len() == EVENT_CAPACITY {
            self.events.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.events.push_back(event);
    }

    fn pop(&mut self) -> Option<SceneLifecycleEvent> {
        self.events.pop_front()
    }

    fn dropped(&self) -> u64 {
        self.dropped
    }
}

#[cfg(any(target_os = "ios", test))]
type SceneEventHandler = Box<dyn FnMut(SceneLifecycleEvent) + Send + 'static>;

#[cfg(any(target_os = "ios", test))]
struct EventHandlerDispatch {
    handler: Option<SceneEventHandler>,
    pending: VecDeque<SceneLifecycleEvent>,
    dispatching: bool,
    dropped: u64,
}

/// Shared callback state for the native observer blocks
#[cfg(any(target_os = "ios", test))]
pub(crate) struct SceneEventHandlerState {
    dispatch: Mutex<EventHandlerDispatch>,
}

#[cfg(any(target_os = "ios", test))]
impl SceneEventHandlerState {
    pub(crate) fn new() -> Self {
        Self {
            dispatch: Mutex::new(EventHandlerDispatch {
                handler: None,
                pending: VecDeque::with_capacity(HANDLER_PENDING_CAPACITY),
                dispatching: false,
                dropped: 0,
            }),
        }
    }

    pub(crate) fn set(&self, handler: impl FnMut(SceneLifecycleEvent) + Send + 'static) {
        let mut dispatch = self
            .dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        dispatch.handler = Some(Box::new(handler));
    }

    pub(crate) fn clear(&self) {
        let mut dispatch = self
            .dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        dispatch.handler = None;
        dispatch.pending.clear();
    }

    fn is_active(&self) -> bool {
        let dispatch = self
            .dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        dispatch.handler.is_some() || dispatch.dispatching
    }

    pub(crate) fn dropped(&self) -> u64 {
        self.dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .dropped
    }

    fn dispatch(&self, event: SceneLifecycleEvent) {
        let should_drain = {
            let mut dispatch = self
                .dispatch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if dispatch.handler.is_none() && !dispatch.dispatching {
                return;
            }
            if dispatch.pending.len() == HANDLER_PENDING_CAPACITY {
                dispatch.pending.pop_front();
                dispatch.dropped = dispatch.dropped.saturating_add(1);
            }
            dispatch.pending.push_back(event);
            if dispatch.dispatching {
                false
            } else {
                dispatch.dispatching = true;
                true
            }
        };
        if !should_drain {
            return;
        }

        let mut handler = {
            let mut dispatch = self
                .dispatch
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(handler) = dispatch.handler.take() else {
                dispatch.pending.clear();
                dispatch.dispatching = false;
                return;
            };
            handler
        };

        loop {
            let event = {
                let mut dispatch = self
                    .dispatch
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                match dispatch.pending.pop_front() {
                    Some(event) => event,
                    None => {
                        dispatch.handler = Some(handler);
                        dispatch.dispatching = false;
                        return;
                    }
                }
            };
            let _ = contain_callback_panic(|| handler(event));
        }
    }
}

#[cfg(any(target_os = "ios", test))]
fn publish_event_if_live(
    events: &Weak<Mutex<EventQueue>>,
    handler: &Weak<SceneEventHandlerState>,
    event: SceneLifecycleEvent,
) -> bool {
    let Some(events) = events.upgrade() else {
        return false;
    };
    let handler_event = handler
        .upgrade()
        .filter(|handler| handler.is_active())
        .map(|_| event.clone());
    events
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .push(event);
    if let Some(event) = handler_event {
        if let Some(handler) = handler.upgrade() {
            handler.dispatch(event);
        }
    }
    true
}

#[derive(Clone, Copy)]
#[cfg(any(target_os = "ios", test))]
enum NativeSceneNotification {
    WillConnect,
    DidDisconnect,
    DidActivate,
    WillDeactivate,
    WillEnterForeground,
    DidEnterBackground,
}

#[cfg(any(target_os = "ios", test))]
impl NativeSceneNotification {
    #[cfg(test)]
    const fn constant_name(self) -> &'static str {
        match self {
            Self::WillConnect => "UISceneWillConnectNotification",
            Self::DidDisconnect => "UISceneDidDisconnectNotification",
            Self::DidActivate => "UISceneDidActivateNotification",
            Self::WillDeactivate => "UISceneWillDeactivateNotification",
            Self::WillEnterForeground => "UISceneWillEnterForegroundNotification",
            Self::DidEnterBackground => "UISceneDidEnterBackgroundNotification",
        }
    }

    const fn event_kind(self) -> SceneLifecycleEventKind {
        match self {
            Self::WillConnect => SceneLifecycleEventKind::WillConnect,
            Self::DidDisconnect => SceneLifecycleEventKind::DidDisconnect,
            Self::DidActivate => SceneLifecycleEventKind::DidActivate,
            Self::WillDeactivate => SceneLifecycleEventKind::WillDeactivate,
            Self::WillEnterForeground => SceneLifecycleEventKind::WillEnterForeground,
            Self::DidEnterBackground => SceneLifecycleEventKind::DidEnterBackground,
        }
    }
}

#[cfg(any(target_os = "ios", test))]
const SCENE_NOTIFICATIONS: [NativeSceneNotification; 6] = [
    NativeSceneNotification::WillConnect,
    NativeSceneNotification::DidDisconnect,
    NativeSceneNotification::DidActivate,
    NativeSceneNotification::WillDeactivate,
    NativeSceneNotification::WillEnterForeground,
    NativeSceneNotification::DidEnterBackground,
];

#[cfg(any(target_os = "ios", test))]
fn contain_callback_panic(callback: impl FnOnce()) -> bool {
    #[cfg(target_os = "ios")]
    {
        ios_runtime::ffi::catch_unwind(callback).is_ok()
    }
    #[cfg(not(target_os = "ios"))]
    {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).is_ok()
    }
}

#[cfg(any(target_os = "ios", test))]
fn validate_root_attachment(
    already_parented: bool,
    loaded_view_has_window: impl FnOnce() -> bool,
) -> Result<()> {
    if already_parented || loaded_view_has_window() {
        Err(Error::new(ErrorKind::InvalidInput))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parented_root_is_rejected_without_loaded_view_inspection() {
        let mut inspected = false;
        let error = validate_root_attachment(true, || {
            inspected = true;
            false
        })
        .unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidInput);
        assert!(!inspected);
    }

    #[test]
    fn window_attached_loaded_root_is_rejected() {
        let error = validate_root_attachment(false, || true).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn root_without_a_window_attached_loaded_view_is_accepted() {
        assert!(validate_root_attachment(false, || false).is_ok());
    }

    fn event(index: u16) -> SceneLifecycleEvent {
        SceneLifecycleEvent {
            session_identifier: SceneSessionIdentifier::from_utf16_code_units(&[index]),
            kind: SceneLifecycleEventKind::DidActivate,
        }
    }

    #[test]
    fn notification_mapping_covers_each_public_scene_transition() {
        let cases = [
            (
                "UISceneWillConnectNotification",
                SceneLifecycleEventKind::WillConnect,
            ),
            (
                "UISceneDidDisconnectNotification",
                SceneLifecycleEventKind::DidDisconnect,
            ),
            (
                "UISceneDidActivateNotification",
                SceneLifecycleEventKind::DidActivate,
            ),
            (
                "UISceneWillDeactivateNotification",
                SceneLifecycleEventKind::WillDeactivate,
            ),
            (
                "UISceneWillEnterForegroundNotification",
                SceneLifecycleEventKind::WillEnterForeground,
            ),
            (
                "UISceneDidEnterBackgroundNotification",
                SceneLifecycleEventKind::DidEnterBackground,
            ),
        ];
        let actual = SCENE_NOTIFICATIONS
            .map(|notification| (notification.constant_name(), notification.event_kind()));
        assert_eq!(actual, cases);
    }

    #[test]
    fn activation_state_mapping_preserves_unknown_native_values() {
        let cases = [
            (-1, SceneActivationState::Unattached),
            (0, SceneActivationState::ForegroundActive),
            (1, SceneActivationState::ForegroundInactive),
            (2, SceneActivationState::Background),
            (3, SceneActivationState::Unknown(3)),
            (-9, SceneActivationState::Unknown(-9)),
        ];
        for (native_value, expected) in cases {
            assert_eq!(
                SceneActivationState::from_native_value(native_value),
                expected
            );
        }
    }

    #[test]
    fn activation_snapshot_keeps_exact_session_identity_with_phase() {
        let units = [b'A' as u16, 0, b'B' as u16];
        let snapshot = SceneActivationSnapshot {
            session_identifier: SceneSessionIdentifier::from_utf16_code_units(&units),
            state: SceneActivationState::ForegroundInactive,
        };
        assert_eq!(snapshot.session_identifier().as_utf16_code_units(), &units);
        assert_eq!(snapshot.state(), SceneActivationState::ForegroundInactive);
    }

    #[test]
    fn queue_drops_the_oldest_event_at_its_fixed_capacity() {
        let mut queue = EventQueue::new();
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.dropped(), 0);
        for index in 0..EVENT_CAPACITY as u16 + 1 {
            queue.push(event(index));
        }
        assert_eq!(queue.events.len(), EVENT_CAPACITY);
        assert_eq!(queue.dropped(), 1);
        assert_eq!(queue.pop(), Some(event(1)));
        assert_eq!(queue.pop(), Some(event(2)));
        for index in 3..EVENT_CAPACITY as u16 + 1 {
            assert_eq!(queue.pop(), Some(event(index)));
        }
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn session_identifier_preserves_nul_and_unpaired_surrogates() {
        let units = [b'A' as u16, 0, 0xD800, b'Z' as u16];
        let identifier = SceneSessionIdentifier::from_utf16_code_units(&units);
        assert_eq!(identifier.as_utf16_code_units(), &units);
        assert_eq!(identifier.to_string_lossless(), None);
        let valid = SceneSessionIdentifier::from_utf16_code_units(&[b'A' as u16, 0, b'B' as u16]);
        assert_eq!(valid.to_string_lossless().as_deref(), Some("A\0B"));
    }

    #[test]
    fn weak_callback_does_not_resurrect_state_after_teardown() {
        let events = Arc::new(Mutex::new(EventQueue::new()));
        let handler = Arc::new(SceneEventHandlerState::new());
        let callback_events = Arc::downgrade(&events);
        let callback_handler = Arc::downgrade(&handler);
        drop(events);
        drop(handler);
        assert!(!publish_event_if_live(
            &callback_events,
            &callback_handler,
            event(1)
        ));
        assert!(callback_events.upgrade().is_none());
        assert!(callback_handler.upgrade().is_none());
    }

    #[test]
    fn direct_handler_receives_events_even_when_poll_queue_drops_oldest() {
        let events = Arc::new(Mutex::new(EventQueue::new()));
        let handler = Arc::new(SceneEventHandlerState::new());
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_by_handler = Arc::clone(&received);
        handler.set(move |event| {
            received_by_handler
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(event);
        });
        let weak_events = Arc::downgrade(&events);
        let weak_handler = Arc::downgrade(&handler);
        let count = EVENT_CAPACITY + 5;

        for index in 0..count as u16 {
            assert!(publish_event_if_live(
                &weak_events,
                &weak_handler,
                event(index)
            ));
        }

        let received = received
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(received.len(), count);
        assert!(received.iter().enumerate().all(|(index, event)| {
            event.session_identifier().as_utf16_code_units() == [index as u16]
        }));
        assert_eq!(events.lock().unwrap().dropped(), 5);
        assert_eq!(handler.dropped(), 0);
    }

    #[test]
    fn no_handler_keeps_poll_delivery_without_callback_backlog() {
        let events = Arc::new(Mutex::new(EventQueue::new()));
        let handler = Arc::new(SceneEventHandlerState::new());
        assert!(publish_event_if_live(
            &Arc::downgrade(&events),
            &Arc::downgrade(&handler),
            event(7)
        ));
        assert_eq!(events.lock().unwrap().pop(), Some(event(7)));
        assert!(!handler.is_active());
        assert_eq!(handler.dropped(), 0);
    }

    #[test]
    fn nested_events_wait_until_the_fn_mut_returns() {
        let handler = Arc::new(SceneEventHandlerState::new());
        let weak_handler = Arc::downgrade(&handler);
        let active = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_active = Arc::clone(&active);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |received_event| {
            assert!(!callback_active.swap(true, std::sync::atomic::Ordering::SeqCst));
            callback_received
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(received_event.session_identifier().as_utf16_code_units()[0]);
            if received_event.session_identifier().as_utf16_code_units() == [0] {
                weak_handler.upgrade().unwrap().dispatch(event(1));
                assert_eq!(
                    callback_received
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .len(),
                    1
                );
            }
            callback_active.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        handler.dispatch(event(0));

        assert_eq!(
            *received
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
            [0, 1]
        );
        assert_eq!(handler.dropped(), 0);
    }

    #[test]
    fn nested_callback_backlog_is_bounded_and_counts_its_own_drops() {
        let handler = Arc::new(SceneEventHandlerState::new());
        let weak_handler = Arc::downgrade(&handler);
        let received = Arc::new(Mutex::new(Vec::new()));
        let callback_received = Arc::clone(&received);
        handler.set(move |received_event| {
            let index = received_event.session_identifier().as_utf16_code_units()[0];
            callback_received
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(index);
            if index == 0 {
                let handler = weak_handler.upgrade().unwrap();
                for nested in 1..=(HANDLER_PENDING_CAPACITY as u16 + 1) {
                    handler.dispatch(event(nested));
                }
            }
        });

        handler.dispatch(event(0));

        let received = received
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert_eq!(received.len(), HANDLER_PENDING_CAPACITY + 1);
        assert_eq!(received[0], 0);
        assert_eq!(received[1], 2);
        assert_eq!(
            received.last(),
            Some(&(HANDLER_PENDING_CAPACITY as u16 + 1))
        );
        assert_eq!(handler.dropped(), 1);
    }

    #[test]
    fn callback_panic_does_not_remove_the_handler() {
        let handler = Arc::new(SceneEventHandlerState::new());
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_calls = Arc::clone(&calls);
        handler.set(move |_| {
            let call = callback_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if call == 0 {
                panic!("expected handler panic");
            }
        });

        handler.dispatch(event(0));
        assert!(handler.is_active());
        handler.dispatch(event(1));

        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn handler_replacement_and_clear_apply_to_later_events() {
        let handler = SceneEventHandlerState::new();
        let old_calls = Arc::new(Mutex::new(0));
        let new_calls = Arc::new(Mutex::new(0));
        let old_handler_calls = Arc::clone(&old_calls);
        handler.set(move |_| {
            *old_handler_calls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) += 1;
        });
        let new_handler_calls = Arc::clone(&new_calls);
        handler.set(move |_| {
            *new_handler_calls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) += 1;
        });
        handler.dispatch(event(0));
        handler.clear();
        handler.dispatch(event(1));

        assert_eq!(*old_calls.lock().unwrap(), 0);
        assert_eq!(*new_calls.lock().unwrap(), 1);
        assert!(!handler.is_active());
    }

    #[test]
    fn callback_panic_is_contained() {
        assert!(!contain_callback_panic(|| panic!(
            "expected callback panic"
        )));
    }
}
