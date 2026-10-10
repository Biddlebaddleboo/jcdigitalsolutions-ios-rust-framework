#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable values and static backend contracts for iCloud identity presence and CloudKit account-status snapshots."]

use core::future::Future;
use framework_core::{Availability, Error};

/// One synchronous observation of whether an iCloud Drive Documents identity token exists.
///
/// This value records only token presence. The API does not return, retain, compare, serialize,
/// stringify, format, or log the opaque native token. `TokenAbsent` does not distinguish why
/// Foundation returned `nil`, and `TokenPresent` does not prove container access, synchronization,
/// or CloudKit account status.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UbiquityIdentitySnapshot {
    /// Foundation returned a non-`nil` iCloud Drive Documents identity token.
    TokenPresent,
    /// Foundation returned `nil`; the reason is not classified by this contract.
    TokenAbsent,
}

impl UbiquityIdentitySnapshot {
    /// Creates a presence-only snapshot from a backend's nullable-token observation.
    pub const fn from_token_presence(token_present: bool) -> Self {
        if token_present {
            Self::TokenPresent
        } else {
            Self::TokenAbsent
        }
    }

    /// Returns whether the backend observed a non-`nil` identity token.
    pub const fn token_present(self) -> bool {
        matches!(self, Self::TokenPresent)
    }
}

/// A statically selected backend for a synchronous iCloud Drive identity-presence snapshot.
///
/// A call performs a fresh, non-prompting observation and returns only the token-presence value.
/// The result may become stale immediately after the call; this trait does not observe identity
/// changes or provide continuity, container access, synchronization, or CloudKit status. Use a
/// concrete backend type; this contract does not support trait-object dispatch.
pub trait UbiquityIdentityBackend: Sized {
    /// Reads a presence-only snapshot without returning or retaining the native token.
    fn snapshot(&self) -> UbiquityIdentitySnapshot;
}

/// The account-availability value returned by a CloudKit account-status snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AccountStatus {
    /// CloudKit cannot determine the iCloud account status.
    CouldNotDetermine,
    /// The iCloud account is available to CloudKit.
    Available,
    /// System restrictions deny access to the iCloud account.
    Restricted,
    /// No iCloud account is configured on the device.
    NoAccount,
    /// The iCloud account is signed in but not ready for CloudKit operations.
    TemporarilyUnavailable,
    /// A newer platform returned an unrecognized fixed-width status value.
    Unknown(i64),
}

/// One owned account-status result with an optional native query error.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AccountStatusSnapshot {
    status: AccountStatus,
    error: Option<Error>,
}

impl AccountStatusSnapshot {
    /// Creates a snapshot from a status and optional framework-owned error.
    pub const fn new(status: AccountStatus, error: Option<Error>) -> Self {
        Self { status, error }
    }

    /// Returns the portable status value.
    pub const fn status(self) -> AccountStatus {
        self.status
    }

    /// Returns the optional query error with its portable kind and optional platform code.
    ///
    /// An error may accompany a status, as in CloudKit's native completion handler. No native
    /// error object or error domain is retained by this value.
    pub const fn error(self) -> Option<Error> {
        self.error
    }
}

/// A compile-time-selected backend for one CloudKit account-status snapshot.
pub trait CloudAccountBackend: Sized {
    /// The backend's concrete status-query future.
    type AccountStatusFuture<'a>: Future<Output = Result<AccountStatusSnapshot, Error>> + 'a
    where
        Self: 'a;

    /// Reports whether the backend API is usable in this target and context.
    fn availability(&self) -> Availability;

    /// Starts one status query on first poll.
    ///
    /// If the query completes while its future remains live, the backend must resolve that future
    /// with exactly one terminal result. Callback-backed implementations must ignore duplicate
    /// completions. Dropping the future delegates cancellation or result abandonment to the
    /// backend; this portable facade promises neither native cancellation nor continued interest.
    fn account_status<'a>(&'a mut self) -> Self::AccountStatusFuture<'a>;
}

/// A thin status-query facade over a caller-owned, statically selected backend.
pub struct CloudAccount<B> {
    backend: B,
}

impl<B: CloudAccountBackend> CloudAccount<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Requests one account-status snapshot through the selected backend.
    ///
    /// The backend's future starts on first poll. A returned status describes only the query time;
    /// it does not observe later account changes or prove access to any database or container data.
    pub fn account_status(&mut self) -> B::AccountStatusFuture<'_> {
        self.backend.account_status()
    }

    /// Borrows the backend for platform-specific controls not modeled by this contract.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for platform-specific controls not modeled by this contract.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

#[cfg(test)]
mod tests {
    use super::{UbiquityIdentityBackend, UbiquityIdentitySnapshot};

    struct FakeBackend(bool);

    impl UbiquityIdentityBackend for FakeBackend {
        fn snapshot(&self) -> UbiquityIdentitySnapshot {
            UbiquityIdentitySnapshot::from_token_presence(self.0)
        }
    }

    fn snapshot<B: UbiquityIdentityBackend>(backend: &B) -> UbiquityIdentitySnapshot {
        backend.snapshot()
    }

    #[test]
    fn nullable_observation_maps_only_to_presence() {
        assert_eq!(
            UbiquityIdentitySnapshot::from_token_presence(true),
            UbiquityIdentitySnapshot::TokenPresent
        );
        assert_eq!(
            UbiquityIdentitySnapshot::from_token_presence(false),
            UbiquityIdentitySnapshot::TokenAbsent
        );
    }

    #[test]
    fn static_fake_backend_reports_only_presence() {
        assert_eq!(
            snapshot(&FakeBackend(true)),
            UbiquityIdentitySnapshot::TokenPresent
        );
        assert_eq!(
            snapshot(&FakeBackend(false)),
            UbiquityIdentitySnapshot::TokenAbsent
        );
    }
}
