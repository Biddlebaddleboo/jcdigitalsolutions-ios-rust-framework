#![deny(missing_docs)]
#![doc = "UIKit scene activity snapshots and lifecycle events for iOS apps"]

#[cfg(any(target_os = "ios", test))]
use std::collections::VecDeque;
#[cfg(test)]
use std::sync::Arc;
#[cfg(any(target_os = "ios", test))]
use std::sync::{Mutex, Weak};

#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use objc2_ui_kit::{UIApplication, UISceneActivationState};

#[cfg(target_os = "ios")]
mod backend;

#[cfg(target_os = "ios")]
pub use backend::IosSceneLifecycleEvents;

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

    fn record(&mut self, state: UISceneActivationState) {
        self.connected_scene_count += 1;
        match state {
            UISceneActivationState::Unattached => self.unattached_count += 1,
            UISceneActivationState::ForegroundActive => self.foreground_active_count += 1,
            UISceneActivationState::ForegroundInactive => self.foreground_inactive_count += 1,
            UISceneActivationState::Background => self.background_count += 1,
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

#[cfg(any(target_os = "ios", test))]
const EVENT_CAPACITY: usize = 32;

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
fn enqueue_if_live(state: &Weak<Mutex<EventQueue>>, event: SceneLifecycleEvent) -> bool {
    let Some(state) = state.upgrade() else {
        return false;
    };
    let mut events = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    events.push(event);
    true
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
        let state = Arc::new(Mutex::new(EventQueue::new()));
        let callback_state = Arc::downgrade(&state);
        drop(state);
        assert!(!enqueue_if_live(&callback_state, event(1)));
        assert!(callback_state.upgrade().is_none());
    }

    #[test]
    fn callback_panic_is_contained() {
        assert!(!contain_callback_panic(|| panic!(
            "expected callback panic"
        )));
    }
}
