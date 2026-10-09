#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![doc = "Point-in-time UIKit scene activity snapshot for iOS apps"]

use ios_runtime::main_thread::MainThread;
use objc2_ui_kit::{UIApplication, UISceneActivationState};

/// Counts scene activation states from one `UIApplication.connectedScenes` read
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneActivitySnapshot {
    connected_scene_count: usize,
    unattached_count: usize,
    foreground_active_count: usize,
    foreground_inactive_count: usize,
    background_count: usize,
    unknown_state_count: usize,
}

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
pub fn snapshot(main_thread: MainThread) -> SceneActivitySnapshot {
    let application = UIApplication::sharedApplication(main_thread.into_objc2());
    let scenes = application.connectedScenes().allObjects();
    let mut snapshot = SceneActivitySnapshot::empty();
    for scene in scenes.iter() {
        snapshot.record(scene.activationState());
    }
    snapshot
}
