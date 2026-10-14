use crate::SceneLifecycleEventKind;

/// Current lifecycle phase for one framework-owned UIKit scene.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IosScenePhase {
    /// UIKit is connecting this scene and its window is being installed.
    Connecting,
    /// UIKit reports this scene is active in the foreground.
    ForegroundActive,
    /// UIKit reports this scene is foreground but inactive.
    ForegroundInactive,
    /// UIKit reports this scene is in the background.
    Background,
    /// UIKit disconnected the scene and its host is being torn down.
    Disconnected,
}

pub(crate) const fn phase_for_event(event: SceneLifecycleEventKind) -> IosScenePhase {
    match event {
        SceneLifecycleEventKind::WillConnect => IosScenePhase::Connecting,
        SceneLifecycleEventKind::DidDisconnect => IosScenePhase::Disconnected,
        SceneLifecycleEventKind::DidActivate => IosScenePhase::ForegroundActive,
        SceneLifecycleEventKind::WillDeactivate | SceneLifecycleEventKind::WillEnterForeground => {
            IosScenePhase::ForegroundInactive
        }
        SceneLifecycleEventKind::DidEnterBackground => IosScenePhase::Background,
    }
}

#[cfg(test)]
mod tests {
    use super::{IosScenePhase, phase_for_event};
    use crate::SceneLifecycleEventKind;

    #[test]
    fn scene_lifecycle_events_map_to_distinct_native_phases() {
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::WillConnect),
            IosScenePhase::Connecting
        );
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::DidActivate),
            IosScenePhase::ForegroundActive
        );
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::WillDeactivate),
            IosScenePhase::ForegroundInactive
        );
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::WillEnterForeground),
            IosScenePhase::ForegroundInactive
        );
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::DidEnterBackground),
            IosScenePhase::Background
        );
        assert_eq!(
            phase_for_event(SceneLifecycleEventKind::DidDisconnect),
            IosScenePhase::Disconnected
        );
    }
}
