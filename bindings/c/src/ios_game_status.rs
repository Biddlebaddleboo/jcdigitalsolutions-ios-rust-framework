use core::panic::AssertUnwindSafe;
use framework_abi::{FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use framework_game::{LocalPlayer, LocalPlayerAuthenticationStatus};
#[cfg(target_os = "ios")]
use ios_game::IosGameCenterBackend;

/// Writes the current Game Center local-player authentication snapshot.
///
/// This reads only B55's `GKLocalPlayer.localPlayer` and `isAuthenticated` path. False does not
/// prove that no account exists; an offline temporary player or uninitialized service can report
/// false. This function does not initialize authentication or present UI.
///
/// # Safety
/// `out_authenticated` must be non-null and point to valid, properly aligned writable memory for
/// one byte for the duration of this synchronous call. The caller must prevent unsynchronized
/// concurrent access to that byte. This function checks only nullness and does not retain the
/// pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_game_status_is_local_player_authenticated(
    out_authenticated: *mut u8,
) -> FrameworkStatus {
    if out_authenticated.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one writable byte at this non-null pointer.
    unsafe { out_authenticated.write(0) };

    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let player = LocalPlayer::new(IosGameCenterBackend::new());
            match player.authentication_status() {
                LocalPlayerAuthenticationStatus::Authenticated => {
                    // SAFETY: The caller supplied one writable byte above.
                    unsafe { out_authenticated.write(1) };
                    FrameworkStatus::OK
                }
                LocalPlayerAuthenticationStatus::NotAuthenticated => FrameworkStatus::OK,
                _ => FrameworkStatus::UNAVAILABLE,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
