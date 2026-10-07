/// A normalized authorization decision suitable for portable capability contracts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum AuthorizationState {
    /// Authorization has not been queried or cannot be classified.
    Unknown = 0,
    /// The platform has not yet asked the user.
    NotDetermined = 1,
    /// The platform or user denied access.
    Denied = 2,
    /// Access is blocked by policy or device restrictions.
    Restricted = 3,
    /// The requested access is authorized.
    Authorized = 4,
}

/// A normalized permission state, including capabilities that require no prompt.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum PermissionState {
    /// Permission has not been queried or cannot be classified.
    Unknown = 0,
    /// The capability does not require user permission.
    NotRequired = 1,
    /// The platform has not yet asked the user.
    NotDetermined = 2,
    /// The user or platform denied access.
    Denied = 3,
    /// Access is blocked by policy or device restrictions.
    Restricted = 4,
    /// The requested permission is granted.
    Granted = 5,
}

/// The semantic cause of a cancellation request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum Cancellation {
    /// The application caller requested cancellation.
    Caller = 1,
    /// An owning or parent operation requested cancellation.
    Parent = 2,
    /// A timeout policy requested cancellation.
    Timeout = 3,
    /// The platform backend is tearing down the operation.
    BackendTeardown = 4,
}
