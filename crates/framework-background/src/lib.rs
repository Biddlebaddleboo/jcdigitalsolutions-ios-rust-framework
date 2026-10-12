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
        validate_task_id(&value)?;
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
/// The callback runs on a backend-owned thread or queue, with no order or main-thread promise, and must return promptly after expiry
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
    /// No finish call is assured after process kill or abort
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

/// One caller-assigned, process-registered background-processing task ID
///
/// The value remains exact and case-sensitive, with no trim or normalization
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ProcessingTaskId(String);

impl ProcessingTaskId {
    /// Creates an ID from a non-empty string with no NUL byte
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_task_id(&value)?;
        Ok(Self(value))
    }

    /// Returns the exact task ID string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a processing task request requires network connectivity
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum NetworkConnectivityRequirement {
    /// Network connectivity is not required by the request
    #[default]
    NotRequired,
    /// Ask the system to run the request only when network connectivity is available
    Required,
}

/// Whether a processing task request requires external power
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ExternalPowerRequirement {
    /// External power is not required by the request
    #[default]
    NotRequired,
    /// Ask the system to run the request only while connected to external power
    Required,
}

/// Typed system conditions for one background-processing task request
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProcessingTaskRequirements {
    network_connectivity: NetworkConnectivityRequirement,
    external_power: ExternalPowerRequirement,
}

impl ProcessingTaskRequirements {
    /// Creates requirements that do not require network connectivity or external power
    pub const fn new() -> Self {
        Self {
            network_connectivity: NetworkConnectivityRequirement::NotRequired,
            external_power: ExternalPowerRequirement::NotRequired,
        }
    }

    /// Returns the network-connectivity requirement
    pub const fn network_connectivity(self) -> NetworkConnectivityRequirement {
        self.network_connectivity
    }

    /// Returns the external-power requirement
    pub const fn external_power(self) -> ExternalPowerRequirement {
        self.external_power
    }

    /// Returns requirements with the requested network-connectivity condition
    pub const fn with_network_connectivity(
        mut self,
        requirement: NetworkConnectivityRequirement,
    ) -> Self {
        self.network_connectivity = requirement;
        self
    }

    /// Returns requirements with the requested external-power condition
    pub const fn with_external_power(mut self, requirement: ExternalPowerRequirement) -> Self {
        self.external_power = requirement;
        self
    }
}

/// One request for a background-processing task under a caller-assigned ID
///
/// The system decides if and when the request runs; no start time or recurrence is promised
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessingTaskRequest {
    task_id: ProcessingTaskId,
    requirements: ProcessingTaskRequirements,
}

impl ProcessingTaskRequest {
    /// Creates a request with explicit system requirements
    pub const fn new(task_id: ProcessingTaskId, requirements: ProcessingTaskRequirements) -> Self {
        Self {
            task_id,
            requirements,
        }
    }

    /// Returns the caller-assigned ID for this request
    pub const fn task_id(&self) -> &ProcessingTaskId {
        &self.task_id
    }

    /// Returns the typed system requirements for this request
    pub const fn requirements(&self) -> ProcessingTaskRequirements {
        self.requirements
    }
}

/// The return value from one background-processing task callback
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProcessingTaskOutcome {
    /// The callback reports its work as complete
    Succeeded,
    /// The callback reports its work as incomplete or failed
    Failed,
}

/// Borrowed data for one synchronous background-processing callback
pub struct ProcessingTaskContext<'a> {
    task_id: &'a ProcessingTaskId,
    expiry: &'a dyn ExpirySignal,
}

impl<'a> ProcessingTaskContext<'a> {
    /// Creates callback data from an ID and a live expiry signal
    pub const fn new(task_id: &'a ProcessingTaskId, expiry: &'a dyn ExpirySignal) -> Self {
        Self { task_id, expiry }
    }

    /// Returns the ID for this run
    pub const fn task_id(&self) -> &ProcessingTaskId {
        self.task_id
    }

    /// Returns the live expiry state for this run
    pub fn is_expired(&self) -> bool {
        self.expiry.is_expired()
    }
}

/// A static backend for background-processing task register, submit, and cancel calls
///
/// `Ok` from submit means only backend acceptance, not a run or time promise
/// The callback is synchronous work, not a `Future`; it must return promptly after expiry
/// The callback runs on a backend-owned thread or queue, with no order or main-thread promise
/// The backend retains each callback while its ID remains registered; no unregister call exists
pub trait ProcessingTaskBackend {
    /// Registers one synchronous work closure for `task_id`
    ///
    /// Register each ID once per app process before launch ends; a native duplicate call can end the app
    /// The backend must catch a Rust panic and finish with failure once
    /// Expiry is a cooperative signal only; it does not stop Rust work
    /// No finish call is assured after process kill or abort
    fn register_processing_task<F>(&self, task_id: &ProcessingTaskId, handler: F) -> Result<()>
    where
        F: for<'a> Fn(ProcessingTaskContext<'a>) -> ProcessingTaskOutcome + Send + Sync + 'static;

    /// Asks the OS to add or replace the pending request for this ID
    ///
    /// Success means accepted by the backend only, not a run time or launch promise
    fn submit_processing_task(&self, request: &ProcessingTaskRequest) -> Result<()>;

    /// Asks the OS to cancel the pending request for this ID
    ///
    /// This does not stop a task closure that has already begun
    fn cancel_processing_task(&self, task_id: &ProcessingTaskId) -> Result<()>;
}

fn validate_task_id(value: &str) -> Result<()> {
    if value.is_empty() || value.contains('\0') {
        Err(Error::new(ErrorKind::InvalidInput))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    struct TestExpiry(Cell<bool>);

    impl ExpirySignal for TestExpiry {
        fn is_expired(&self) -> bool {
            self.0.get()
        }
    }

    #[test]
    fn processing_task_id_preserves_exact_string_and_rejects_invalid_values() {
        let id = ProcessingTaskId::new(" com.Example.Processing ").unwrap();
        assert_eq!(id.as_str(), " com.Example.Processing ");
        assert_eq!(
            ProcessingTaskId::new("").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            ProcessingTaskId::new("com.example\0processing")
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn app_refresh_id_uses_the_same_identifier_validation() {
        assert_eq!(
            AppRefreshTaskId::new("").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(
            AppRefreshTaskId::new("refresh\0task").unwrap_err().kind(),
            ErrorKind::InvalidInput
        );
    }

    #[test]
    fn processing_requirements_are_typed_and_default_to_unrestricted() {
        let defaults = ProcessingTaskRequirements::default();
        assert_eq!(
            defaults.network_connectivity(),
            NetworkConnectivityRequirement::NotRequired
        );
        assert_eq!(
            defaults.external_power(),
            ExternalPowerRequirement::NotRequired
        );

        let constrained = defaults
            .with_network_connectivity(NetworkConnectivityRequirement::Required)
            .with_external_power(ExternalPowerRequirement::Required);
        assert_eq!(
            constrained.network_connectivity(),
            NetworkConnectivityRequirement::Required
        );
        assert_eq!(
            constrained.external_power(),
            ExternalPowerRequirement::Required
        );
    }

    #[test]
    fn processing_request_keeps_its_id_and_requirements() {
        let id = ProcessingTaskId::new("com.example.maintenance").unwrap();
        let requirements = ProcessingTaskRequirements::new()
            .with_network_connectivity(NetworkConnectivityRequirement::Required);
        let request = ProcessingTaskRequest::new(id.clone(), requirements);
        assert_eq!(request.task_id(), &id);
        assert_eq!(request.requirements(), requirements);
    }

    #[test]
    fn processing_context_reads_live_expiry_without_owning_the_task() {
        let id = ProcessingTaskId::new("com.example.maintenance").unwrap();
        let expiry = TestExpiry(Cell::new(false));
        let context = ProcessingTaskContext::new(&id, &expiry);
        assert_eq!(context.task_id(), &id);
        assert!(!context.is_expired());
        expiry.0.set(true);
        assert!(context.is_expired());
    }
}
