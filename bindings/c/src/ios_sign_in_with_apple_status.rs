use core::ffi::c_void;
#[cfg(target_os = "ios")]
use core::panic::AssertUnwindSafe;
#[cfg(target_os = "ios")]
use framework_abi::catch_unwind_status;
use framework_abi::{FrameworkStatus, FrameworkStr};

#[cfg(target_os = "ios")]
use ios_sign_in_with_apple_status::{CredentialState, CredentialStateSnapshot};

/// Signed fixed-width value for one Sign in with Apple credential state.
pub type FrameworkIosSignInWithAppleCredentialState = i64;

/// Revoked credential state.
pub const FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED:
    FrameworkIosSignInWithAppleCredentialState = 0;
/// Authorized credential state.
pub const FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED:
    FrameworkIosSignInWithAppleCredentialState = 1;
/// Credential state with no established Sign in with Apple relationship.
pub const FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND:
    FrameworkIosSignInWithAppleCredentialState = 2;
/// Credential state for an app transferred to another team.
pub const FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED:
    FrameworkIosSignInWithAppleCredentialState = 3;

/// One terminal callback for an accepted Sign in with Apple credential-state query.
pub type FrameworkIosSignInWithAppleCredentialStateCompletion = Option<
    unsafe extern "C" fn(
        context: *mut c_void,
        credential_state: FrameworkIosSignInWithAppleCredentialState,
        had_error: u8,
    ),
>;

fn valid_user_id_span(value: FrameworkStr) -> bool {
    matches!(
        usize::try_from(value.length()),
        Ok(length) if length != 0 && length <= isize::MAX as usize && !value.data().is_null()
    )
}

unsafe fn input_user_id<'a>(value: FrameworkStr) -> Result<&'a str, FrameworkStatus> {
    if !valid_user_id_span(value) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C host keeps the non-empty span readable and unchanged through this call.
    unsafe { value.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

#[cfg(target_os = "ios")]
fn credential_state_raw(value: CredentialState) -> i64 {
    match value {
        CredentialState::Revoked => FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_REVOKED,
        CredentialState::Authorized => FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_AUTHORIZED,
        CredentialState::NotFound => FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_NOT_FOUND,
        CredentialState::Transferred => {
            FRAMEWORK_IOS_SIGN_IN_WITH_APPLE_CREDENTIAL_STATE_TRANSFERRED
        }
        CredentialState::Unknown(raw) => raw,
    }
}

#[cfg(target_os = "ios")]
fn deliver_completion(
    completion: unsafe extern "C" fn(*mut c_void, FrameworkIosSignInWithAppleCredentialState, u8),
    context: usize,
    snapshot: CredentialStateSnapshot,
) {
    // SAFETY: F21's host contract keeps context valid through this callback return and forbids
    // unwinding across C. B78 supplies one Rust-owned completion with scalar snapshot values.
    unsafe {
        completion(
            context as *mut c_void,
            credential_state_raw(snapshot.state),
            u8::from(snapshot.had_error),
        )
    };
}

/// Starts one query for a caller-supplied, previously obtained Sign in with Apple user ID.
///
/// The ID is borrowed only through this call. On an accepted iOS call, the C completion runs
/// once when AuthenticationServices supplies its result, on an unspecified queue and possibly
/// before this function returns. There is no cancel, handle, or later result-poll API.
///
/// # Safety
/// `user_id` must contain readable, unchanged, non-empty UTF-8 bytes through return. A non-null
/// completion must remain callable, must not unwind across C, and its context must remain valid
/// until the completion returns. The callback may run inline or on any queue.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_sign_in_with_apple_credential_state_start(
    user_id: FrameworkStr,
    completion: FrameworkIosSignInWithAppleCredentialStateCompletion,
    context: *mut c_void,
) -> FrameworkStatus {
    let Some(_completion) = completion else {
        return FrameworkStatus::INVALID_ARGUMENT;
    };
    // SAFETY: The caller upholds the FrameworkStr pointer, length, UTF-8, and lifetime contract.
    let user_id = match unsafe { input_user_id(user_id) } {
        Ok(user_id) => user_id,
        Err(status) => return status,
    };

    #[cfg(target_os = "ios")]
    {
        let context = context as usize;
        catch_unwind_status(AssertUnwindSafe(|| {
            ::ios_sign_in_with_apple_status::get_credential_state(user_id, move |snapshot| {
                deliver_completion(_completion, context, snapshot);
            });
            FrameworkStatus::OK
        }))
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (user_id, context);
        FrameworkStatus::UNSUPPORTED
    }
}
