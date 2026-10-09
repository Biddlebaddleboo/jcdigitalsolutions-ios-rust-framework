#![cfg(target_os = "ios")]
#![deny(unsafe_op_in_unsafe_fn)]

use block2::RcBlock;
use objc2_authentication_services::{
    ASAuthorizationAppleIDProvider, ASAuthorizationAppleIDProviderCredentialState,
};
use objc2_foundation::{NSError, NSString};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, PoisonError};

/// Credential state for one caller-supplied prior Sign in with Apple user ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialState {
    Revoked,
    Authorized,
    NotFound,
    Transferred,
    Unknown(i64),
}

/// Rust-owned state and independent presence of an `NSError` from the query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CredentialStateSnapshot {
    pub state: CredentialState,
    pub had_error: bool,
}

type Completion = Box<dyn FnOnce(CredentialStateSnapshot) + Send + 'static>;
type CompletionCell = Arc<Mutex<Option<Completion>>>;

/// Starts a credential-state query for a previously obtained Sign in with Apple user ID.
///
/// `completion` may run on any queue and can run before this function returns. The API has no
/// cancellation operation; returning from or abandoning the caller's work does not stop the
/// AuthenticationServices query. The user ID is copied to an `NSString` and is not returned.
pub fn get_credential_state(
    user_id: &str,
    completion: impl FnOnce(CredentialStateSnapshot) + Send + 'static,
) {
    let provider = unsafe { ASAuthorizationAppleIDProvider::new() };
    let user_id = NSString::from_str(user_id);
    let completion: CompletionCell = Arc::new(Mutex::new(Some(Box::new(completion))));
    let callback_state = Arc::clone(&completion);
    let handler = RcBlock::new(
        move |native_state: ASAuthorizationAppleIDProviderCredentialState, error: *mut NSError| {
            let result = CredentialStateSnapshot {
                state: match native_state.0 {
                    0 => CredentialState::Revoked,
                    1 => CredentialState::Authorized,
                    2 => CredentialState::NotFound,
                    3 => CredentialState::Transferred,
                    raw => CredentialState::Unknown(raw as i64),
                },
                had_error: !error.is_null(),
            };
            let callback = {
                let mut slot = callback_state
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                slot.take()
            };
            if let Some(callback) = callback {
                let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
            }
        },
    );

    // SAFETY: The user ID is a retained immutable NSString for this call. `handler` is an
    // escaping heap-owned block. This API reports asynchronously through its completion block;
    // the callback state owns only a Send + 'static closure and copies native scalar values.
    unsafe {
        provider.getCredentialStateForUserID_completion(&user_id, &handler);
    }
}
