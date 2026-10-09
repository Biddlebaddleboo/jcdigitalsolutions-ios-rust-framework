#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable local-player authentication-status values and a static backend contract."]

use framework_core::Availability;

/// The point-in-time authentication value reported for a local game-service player.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LocalPlayerAuthenticationStatus {
    /// The backend reports that the local player is authenticated.
    Authenticated,
    /// The backend reports that the local player is not authenticated.
    ///
    /// This does not distinguish a signed-out account from an uninitialized game service.
    NotAuthenticated,
}

/// A statically selected backend for a local player's current authentication status.
pub trait LocalPlayerAuthenticationBackend {
    /// Reports whether this backend API is supported for the current target.
    ///
    /// This does not query account state or validate service configuration and entitlements.
    fn availability(&self) -> Availability;

    /// Reads the backend's current local-player authentication value without requesting sign-in.
    fn authentication_status(&self) -> LocalPlayerAuthenticationStatus;
}

/// A thin local-player status facade over a caller-owned backend.
pub struct LocalPlayer<B> {
    backend: B,
}

impl<B: LocalPlayerAuthenticationBackend> LocalPlayer<B> {
    /// Creates a status facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports whether the backend API exists in the current target and configuration.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Returns one synchronous, point-in-time authentication value without requesting sign-in.
    pub fn authentication_status(&self) -> LocalPlayerAuthenticationStatus {
        self.backend.authentication_status()
    }

    /// Borrows the selected backend for platform-specific controls not modeled by this contract.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the selected backend for platform-specific controls not modeled here.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}
