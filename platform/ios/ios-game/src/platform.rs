use framework_core::Availability;
use framework_game::{LocalPlayerAuthenticationBackend, LocalPlayerAuthenticationStatus};
use objc2_game_kit::GKLocalPlayer;

/// A stateless, non-prompting Game Center local-player status backend.
#[derive(Clone, Copy, Debug, Default)]
pub struct IosGameCenterBackend;

impl IosGameCenterBackend {
    /// Creates a backend that reads only `GKLocalPlayer.isAuthenticated`.
    pub const fn new() -> Self {
        Self
    }
}

impl LocalPlayerAuthenticationBackend for IosGameCenterBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn authentication_status(&self) -> LocalPlayerAuthenticationStatus {
        // SAFETY: `GKLocalPlayer.localPlayer` is the public nonnull shared-instance accessor. The
        // retained object remains local to this synchronous call and is not exposed or transferred.
        let player = unsafe { GKLocalPlayer::localPlayer() };
        // SAFETY: `player` is a live retained `GKLocalPlayer`; this invokes only its read-only
        // Boolean `isAuthenticated` getter and does not install or invoke an authentication handler.
        if unsafe { player.isAuthenticated() } {
            LocalPlayerAuthenticationStatus::Authenticated
        } else {
            LocalPlayerAuthenticationStatus::NotAuthenticated
        }
    }
}
