#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable app-refresh values and a static backend contract"]

extern crate alloc;

use alloc::string::String;
use framework_core::{Error, ErrorKind, Result};

/// One caller-assigned app-refresh task ID
///
/// The value remains exact and case-sensitive, with no trim or normalization
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AppRefreshTaskId(String);

impl AppRefreshTaskId {
    /// Creates an ID from a non-empty string with no NUL byte
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() || value.contains('\0') {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self(value))
    }

    /// Returns the exact task ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One request for an app-refresh task under a caller-assigned ID.
///
/// No time, interval, or run-count field is present. The OS picks if and when a run can start
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppRefreshRequest {
    task_id: AppRefreshTaskId,
}

impl AppRefreshRequest {
    /// Creates one request for `task_id`
    pub const fn new(task_id: AppRefreshTaskId) -> Self {
        Self { task_id }
    }

    /// Returns the caller-assigned ID for this request
    pub const fn task_id(&self) -> &AppRefreshTaskId {
        &self.task_id
    }
}

/// The return value from one app-refresh task callback
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AppRefreshOutcome {
    /// The callback reports its work as complete
    Succeeded,
    /// The callback reports its work as incomplete or failed
    Failed,
}

/// A read-only signal for an app-refresh task's expiry
pub trait ExpirySignal {
    /// Returns true after the OS asks this run to stop
    fn is_expired(&self) -> bool;
}

/// Borrowed data for one synchronous app-refresh callback
pub struct AppRefreshContext<'a> {
    task_id: &'a AppRefreshTaskId,
    expiry: &'a dyn ExpirySignal,
}

impl<'a> AppRefreshContext<'a> {
    /// Creates callback data from an ID and a live expiry signal
    pub const fn new(task_id: &'a AppRefreshTaskId, expiry: &'a dyn ExpirySignal) -> Self {
        Self { task_id, expiry }
    }

    /// Returns the ID for this run
    pub const fn task_id(&self) -> &AppRefreshTaskId {
        self.task_id
    }

    /// Returns the live expiry state for this run
    pub fn is_expired(&self) -> bool {
        self.expiry.is_expired()
    }
}

/// A static backend for app-refresh task register, submit, and cancel calls
///
/// `Ok` from submit means only that the backend accepted the request, not that a run will start
/// The scheduler controls the run time and may launch one accepted request zero or one time
/// Submit a new request for another run
/// The callback is a synchronous work closure, not a `Future`; no async work is awaited
/// The callback runs on a backend-owned thread or queue and must return promptly after expiry
/// The backend retains the closure while the ID remains registered; no unregister call exists
pub trait AppRefreshBackend {
    /// Registers one synchronous work closure for `task_id`
    ///
    /// Register each ID once per app process before launch ends; a native duplicate call can end the app
    /// The backend retains the closure while the ID remains registered; no unregister call exists
    /// The backend owns each native task and calls its finish method once after the closure returns
    /// A normal return maps `Succeeded` or `Failed` to the native result, unless expiry wins the atomic race
    /// The backend must catch a Rust panic at the native boundary and finish with failure
    /// Expiry is a cooperative atomic signal only; it does not stop Rust work or finish the native task
    /// If the closure ignores expiry and does not return, the OS may end the app and no finish call is assured
    /// No executor, global Rust registry, or future wait is part of this contract
    fn register_app_refresh<F>(&self, task_id: &AppRefreshTaskId, handler: F) -> Result<()>
    where
        F: for<'a> Fn(AppRefreshContext<'a>) -> AppRefreshOutcome + Send + Sync + 'static;

    /// Asks the OS to add or replace the pending request for this ID
    ///
    /// Success means accepted by the backend only, not a run time or launch promise
    fn submit_app_refresh(&self, request: &AppRefreshRequest) -> Result<()>;

    /// Asks the OS to cancel the pending request for this ID
    ///
    /// This does not stop a task closure that has already begun
    fn cancel_app_refresh(&self, task_id: &AppRefreshTaskId) -> Result<()>;
}
