#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable local user-presence authentication and App Tracking Transparency status contracts."]

use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// A requested system-mediated local authentication policy.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AuthenticationPolicy {
    /// Requires biometrics and does not permit local-device-credential fallback.
    BiometricsOnly,
    /// Allows the platform's biometrics or local-device-credential policy.
    DeviceOwner,
}

/// One borrowed request for a system-mediated local authentication prompt.
///
/// The reason is preserved exactly as supplied. The caller should provide a short explanation in
/// the user's current language; the backend must keep no Rust borrow after the returned operation
/// completes or is dropped.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AuthenticationRequest<'a> {
    policy: AuthenticationPolicy,
    reason: &'a str,
}

impl<'a> AuthenticationRequest<'a> {
    /// Creates a request with one policy and non-empty, non-whitespace reason text.
    pub fn new(policy: AuthenticationPolicy, reason: &'a str) -> Result<Self, AuthenticationError> {
        if reason.trim().is_empty() {
            return Err(AuthenticationError::InvalidReason);
        }
        Ok(Self { policy, reason })
    }

    /// Returns the requested platform policy.
    pub const fn policy(self) -> AuthenticationPolicy {
        self.policy
    }

    /// Returns the caller-borrowed reason text without normalization.
    pub const fn reason(self) -> &'a str {
        self.reason
    }
}

/// A stable local-authentication error with an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AuthenticationError {
    /// The request reason is empty or contains only whitespace.
    InvalidReason,
    /// The selected backend returned a framework error.
    Backend(Error),
}

impl AuthenticationError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidReason => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::InvalidReason => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A statically selected backend for one-shot local authentication.
///
/// Availability is synchronous and may block, but must not display a prompt. Authentication
/// begins no earlier than the first poll of its returned future. A successful result means only
/// that the platform reported success for the requested policy at that time; it is not identity
/// proof or a reusable credential. If a pending future is dropped, the backend must request native
/// cancellation where supported and suppress its Rust result. Native callback state must remain
/// safe through cancellation races and be released exactly once. No executor or `Send` requirement
/// is imposed.
pub trait AuthenticationBackend {
    /// Reports whether one policy is usable in the current context without prompting.
    fn availability(&self, policy: AuthenticationPolicy) -> Availability;

    /// The future type for one explicit local-authentication operation.
    type AuthenticateFuture<'a>: Future<Output = Result<(), AuthenticationError>> + 'a
    where
        Self: 'a;

    /// Evaluates the supplied policy when the returned future is first polled.
    ///
    /// A backend may copy the borrowed reason text if native UI or callback state must outlive
    /// the request borrow. Dropping a pending future abandons its Rust result and requests native
    /// cancellation where supported; it does not promise dismissal of system UI already shown.
    fn authenticate<'a>(
        &'a mut self,
        request: AuthenticationRequest<'a>,
    ) -> Self::AuthenticateFuture<'a>;
}

/// A thin facade over caller-owned, statically selected authentication-backend state.
pub struct Authenticator<B> {
    backend: B,
}

/// The current App Tracking Transparency status for the calling app.
///
/// This enum is specific to App Tracking Transparency. It does not represent general privacy,
/// consent, data-access, advertising-identifier, or account authorization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AppTrackingAuthorizationStatus {
    /// The app cannot determine the calling app's authorization status for tracking-related data.
    ///
    /// Apple also returns this status before the device receives an authorization request. It is
    /// neither approval nor denial.
    NotDetermined,
    /// Authorization to access tracking-related data is restricted.
    ///
    /// Apple may return this status when the restriction is managed by the system. It is distinct
    /// from a user denial.
    Restricted,
    /// The user denies authorization to access app-related data that may be used to track the user
    /// or device.
    Denied,
    /// The user authorizes access to app-related data that may be used to track the user or device.
    ///
    /// This reports only the App Tracking Transparency status; it is not a general permission or
    /// legal determination.
    Authorized,
    /// The backend received a native status value this version does not recognize.
    ///
    /// Backends must preserve unrecognized native values as this status rather than treating them
    /// as authorization.
    Unknown,
}

/// A statically selected backend for a non-prompting App Tracking Transparency status query.
///
/// This synchronous query reads the current status for the calling app only. It must not request
/// authorization, display a prompt, access an identifier, perform tracking, or infer authorization
/// for any privacy capability beyond App Tracking Transparency. A native status this version does
/// not recognize must be returned as [`AppTrackingAuthorizationStatus::Unknown`].
pub trait AppTrackingAuthorizationBackend {
    /// Reads the current App Tracking Transparency status for the calling app without prompting.
    fn status(&self) -> AppTrackingAuthorizationStatus;
}

impl<B: AuthenticationBackend> Authenticator<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports policy-specific availability without displaying a prompt.
    pub fn availability(&self, policy: AuthenticationPolicy) -> Availability {
        self.backend.availability(policy)
    }

    /// Explicitly evaluates one local-authentication request.
    ///
    /// The operation starts on first poll. Success reports only the platform policy result at
    /// that time. Dropping a pending future requests native cancellation where supported and
    /// suppresses its Rust result.
    pub async fn authenticate<'a>(
        &'a mut self,
        request: AuthenticationRequest<'a>,
    ) -> Result<(), AuthenticationError> {
        self.backend.authenticate(request).await
    }

    /// Borrows the selected backend.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the selected backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tracking_tests {
    use super::{AppTrackingAuthorizationBackend, AppTrackingAuthorizationStatus};

    struct FixedBackend(AppTrackingAuthorizationStatus);

    impl AppTrackingAuthorizationBackend for FixedBackend {
        fn status(&self) -> AppTrackingAuthorizationStatus {
            self.0
        }
    }

    #[test]
    fn tracking_status_values_are_distinct() {
        let values = [
            AppTrackingAuthorizationStatus::NotDetermined,
            AppTrackingAuthorizationStatus::Restricted,
            AppTrackingAuthorizationStatus::Denied,
            AppTrackingAuthorizationStatus::Authorized,
            AppTrackingAuthorizationStatus::Unknown,
        ];
        for (index, value) in values.iter().enumerate() {
            assert!(values[index + 1..].iter().all(|other| value != other));
        }
    }

    #[test]
    fn tracking_backend_preserves_each_status_value() {
        let values = [
            AppTrackingAuthorizationStatus::NotDetermined,
            AppTrackingAuthorizationStatus::Restricted,
            AppTrackingAuthorizationStatus::Denied,
            AppTrackingAuthorizationStatus::Authorized,
            AppTrackingAuthorizationStatus::Unknown,
        ];
        for value in values {
            assert_eq!(FixedBackend(value).status(), value);
        }
    }
}
