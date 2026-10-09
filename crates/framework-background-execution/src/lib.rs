#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, static contract for a bounded operating-system background-execution lease."]

pub use framework_core::{Error, ErrorKind, Result};

/// Cooperative signal that a native background-execution lease has expired.
///
/// This signal does not interrupt work, extend a deadline, guarantee completion, or request a
/// future launch. Callers must check it between bounded units of work and stop promptly when it
/// becomes true.
pub trait ExpirySignal {
    /// Returns whether the native lease has expired.
    fn is_expired(&self) -> bool;
}

/// A lease that exposes expiry and an explicit end operation.
///
/// Backends may also end a native lease from its expiry callback or when the lease is dropped.
/// Applications should still call [`BackgroundExecutionLease::end`] as soon as their work is done.
#[must_use = "a background-execution lease should be ended explicitly"]
pub trait BackgroundExecutionLease {
    /// The backend-specific cooperative expiry signal.
    type Expiry: ExpirySignal;

    /// Returns the signal used to check for native lease expiry.
    fn expiry(&self) -> &Self::Expiry;

    /// Ends the lease and consumes this handle.
    fn end(self);
}

/// Static backend contract for beginning one bounded background-execution lease.
pub trait BackgroundExecutionBackend {
    /// Context required by the native API, such as a main-thread proof.
    type Context;
    /// Lease type returned by this backend.
    type Lease: BackgroundExecutionLease;

    /// Begins one native lease or returns a portable error.
    ///
    /// Success does not promise a duration, continued execution, or work completion.
    fn begin(&self, context: Self::Context) -> Result<Self::Lease>;
}

/// Zero-policy facade that statically forwards calls to one selected backend.
pub struct BackgroundExecution<B> {
    backend: B,
}

impl<B> BackgroundExecution<B> {
    /// Wraps a backend without allocation or runtime backend selection.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Begins one lease through the selected backend.
    pub fn begin(&self, context: B::Context) -> Result<B::Lease>
    where
        B: BackgroundExecutionBackend,
    {
        self.backend.begin(context)
    }
}
