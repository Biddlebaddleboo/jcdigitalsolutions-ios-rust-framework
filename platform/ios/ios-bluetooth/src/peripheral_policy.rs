use framework_bluetooth::{BluetoothPeripheralError, BluetoothPeripheralState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManagerAvailability {
    PoweredOn,
    PoweredOff,
    Unauthorized,
    Unsupported,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PeripheralAction {
    None,
    AddService,
    StartAdvertising,
    RemoveService,
    ForgetNativeService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PeripheralCleanup {
    pub(crate) stop_advertising: bool,
    pub(crate) remove_service: bool,
}

/// One caller-owned service publication and advertisement lifecycle independent of CoreBluetooth.
pub(crate) struct PeripheralPolicy {
    requested: bool,
    service_added: bool,
    add_pending: bool,
    advertising_pending: bool,
    quarantined: bool,
    state: BluetoothPeripheralState,
    error: Option<BluetoothPeripheralError>,
}

impl PeripheralPolicy {
    pub(crate) const fn new() -> Self {
        Self {
            requested: false,
            service_added: false,
            add_pending: false,
            advertising_pending: false,
            quarantined: false,
            state: BluetoothPeripheralState::Idle,
            error: None,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothPeripheralState {
        self.state
    }

    pub(crate) const fn error(&self) -> Option<BluetoothPeripheralError> {
        self.error
    }

    pub(crate) const fn service_added(&self) -> bool {
        self.service_added
    }

    pub(crate) const fn has_pending_callback(&self) -> bool {
        self.add_pending || self.advertising_pending
    }

    pub(crate) fn start(&mut self) -> Result<(), BluetoothPeripheralError> {
        if self.requested || self.has_pending_callback() || self.quarantined {
            return Err(BluetoothPeripheralError::AlreadyActive);
        }
        self.requested = true;
        self.error = None;
        self.state = BluetoothPeripheralState::Starting;
        Ok(())
    }

    pub(crate) fn manager_update(&mut self, availability: ManagerAvailability) -> PeripheralAction {
        if !self.requested {
            match availability {
                ManagerAvailability::PoweredOff if self.service_added => {
                    self.service_added = false;
                    return PeripheralAction::RemoveService;
                }
                ManagerAvailability::PoweredOff => {}
                ManagerAvailability::PoweredOn
                    if self.service_added && !self.advertising_pending =>
                {
                    return PeripheralAction::RemoveService;
                }
                _ => {}
            }
            return PeripheralAction::None;
        }
        if self.has_pending_callback() {
            if availability != ManagerAvailability::PoweredOn {
                self.requested = false;
                self.quarantined = true;
                self.fail(
                    BluetoothPeripheralState::Unavailable,
                    BluetoothPeripheralError::Unavailable,
                );
                if availability == ManagerAvailability::PoweredOff {
                    let forget_native_service = self.advertising_pending && self.service_added;
                    self.service_added = false;
                    if forget_native_service {
                        return PeripheralAction::ForgetNativeService;
                    }
                }
            }
            return PeripheralAction::None;
        }
        match availability {
            ManagerAvailability::PoweredOn => {
                if self.state == BluetoothPeripheralState::Advertising {
                    return PeripheralAction::None;
                }
                self.error = None;
                if self.service_added {
                    self.advertising_pending = true;
                    self.state = BluetoothPeripheralState::StartingAdvertising;
                    PeripheralAction::StartAdvertising
                } else {
                    self.add_pending = true;
                    self.state = BluetoothPeripheralState::PublishingService;
                    PeripheralAction::AddService
                }
            }
            ManagerAvailability::PoweredOff => {
                let forget_native_service = self.service_added;
                self.service_added = false;
                self.error = None;
                self.state = BluetoothPeripheralState::WaitingForPower;
                if forget_native_service {
                    PeripheralAction::ForgetNativeService
                } else {
                    PeripheralAction::None
                }
            }
            ManagerAvailability::Unauthorized => {
                self.error = Some(BluetoothPeripheralError::Unauthorized);
                self.state = BluetoothPeripheralState::Unauthorized;
                PeripheralAction::None
            }
            ManagerAvailability::Unsupported => {
                self.requested = false;
                self.error = Some(BluetoothPeripheralError::Unsupported);
                self.state = BluetoothPeripheralState::Unsupported;
                PeripheralAction::None
            }
            ManagerAvailability::Other => {
                if self.service_added {
                    self.requested = false;
                    self.fail(
                        BluetoothPeripheralState::Unavailable,
                        BluetoothPeripheralError::Unavailable,
                    );
                } else {
                    self.error = None;
                    self.state = BluetoothPeripheralState::Starting;
                }
                PeripheralAction::None
            }
        }
    }

    pub(crate) fn did_add_service(&mut self, had_error: bool) -> PeripheralAction {
        if !self.add_pending {
            return PeripheralAction::None;
        }
        self.add_pending = false;
        if had_error {
            if self.requested {
                self.requested = false;
                self.fail(
                    BluetoothPeripheralState::Failed,
                    BluetoothPeripheralError::NativeFailure,
                );
            }
            self.release_quarantine_if_idle();
            return PeripheralAction::None;
        }
        self.service_added = true;
        if !self.requested {
            self.state = BluetoothPeripheralState::Stopped;
            self.release_quarantine_if_idle();
            return PeripheralAction::RemoveService;
        }
        self.error = None;
        self.advertising_pending = true;
        self.state = BluetoothPeripheralState::StartingAdvertising;
        PeripheralAction::StartAdvertising
    }

    pub(crate) fn did_start_advertising(&mut self, had_error: bool) -> PeripheralAction {
        if !self.advertising_pending {
            return PeripheralAction::None;
        }
        self.advertising_pending = false;
        if !self.requested {
            if self.state != BluetoothPeripheralState::Failed
                && self.state != BluetoothPeripheralState::Unavailable
            {
                self.state = BluetoothPeripheralState::Stopped;
            }
            self.release_quarantine_if_idle();
            return if self.service_added {
                PeripheralAction::RemoveService
            } else {
                PeripheralAction::None
            };
        }
        if had_error {
            self.requested = false;
            self.fail(
                BluetoothPeripheralState::Failed,
                BluetoothPeripheralError::NativeFailure,
            );
            PeripheralAction::RemoveService
        } else {
            self.error = None;
            self.state = BluetoothPeripheralState::Advertising;
            PeripheralAction::None
        }
    }

    pub(crate) fn stop(&mut self) -> PeripheralCleanup {
        let cleanup = PeripheralCleanup {
            stop_advertising: self.state == BluetoothPeripheralState::Advertising
                || self.advertising_pending,
            remove_service: self.service_added && !self.advertising_pending,
        };
        self.requested = false;
        self.state = BluetoothPeripheralState::Stopped;
        self.quarantined |= self.has_pending_callback();
        cleanup
    }

    pub(crate) fn service_removed(&mut self) {
        self.service_added = false;
    }

    pub(crate) fn callback_panicked(&mut self) {
        self.requested = false;
        self.quarantined = true;
        self.fail(
            BluetoothPeripheralState::Unavailable,
            BluetoothPeripheralError::Unavailable,
        );
    }

    fn fail(&mut self, state: BluetoothPeripheralState, error: BluetoothPeripheralError) {
        self.state = state;
        self.error = Some(error);
    }

    fn release_quarantine_if_idle(&mut self) {
        if !self.has_pending_callback() {
            self.quarantined = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_waits_for_power_then_publishes_before_advertising() {
        let mut policy = PeripheralPolicy::new();
        assert_eq!(policy.start(), Ok(()));
        assert_eq!(policy.state(), BluetoothPeripheralState::Starting);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            PeripheralAction::None
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::WaitingForPower);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::PublishingService);
        assert_eq!(
            policy.did_add_service(false),
            PeripheralAction::StartAdvertising
        );
        assert_eq!(
            policy.state(),
            BluetoothPeripheralState::StartingAdvertising
        );
        assert_eq!(policy.did_start_advertising(false), PeripheralAction::None);
        assert_eq!(policy.state(), BluetoothPeripheralState::Advertising);
        assert_eq!(policy.error(), None);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::None
        );
    }

    #[test]
    fn authorization_and_unsupported_roles_are_distinct() {
        let mut unauthorized = PeripheralPolicy::new();
        unauthorized.start().unwrap();
        assert_eq!(
            unauthorized.manager_update(ManagerAvailability::Unauthorized),
            PeripheralAction::None
        );
        assert_eq!(unauthorized.state(), BluetoothPeripheralState::Unauthorized);
        assert_eq!(
            unauthorized.error(),
            Some(BluetoothPeripheralError::Unauthorized)
        );
        assert_eq!(
            unauthorized.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );

        let mut unsupported = PeripheralPolicy::new();
        unsupported.start().unwrap();
        unsupported.manager_update(ManagerAvailability::Unsupported);
        assert_eq!(unsupported.state(), BluetoothPeripheralState::Unsupported);
        assert_eq!(
            unsupported.error(),
            Some(BluetoothPeripheralError::Unsupported)
        );
        assert_eq!(
            unsupported.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::None
        );
    }

    #[test]
    fn duplicate_start_is_rejected_until_stopped() {
        let mut policy = PeripheralPolicy::new();
        assert_eq!(policy.start(), Ok(()));
        assert_eq!(policy.start(), Err(BluetoothPeripheralError::AlreadyActive));
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
        assert_eq!(policy.start(), Err(BluetoothPeripheralError::AlreadyActive));
    }

    #[test]
    fn service_and_advertisement_failures_are_terminal_and_normalized() {
        let mut add_failed = PeripheralPolicy::new();
        add_failed.start().unwrap();
        add_failed.manager_update(ManagerAvailability::PoweredOn);
        assert_eq!(add_failed.did_add_service(true), PeripheralAction::None);
        assert_eq!(add_failed.state(), BluetoothPeripheralState::Failed);
        assert_eq!(
            add_failed.error(),
            Some(BluetoothPeripheralError::NativeFailure)
        );

        let mut advertising_failed = PeripheralPolicy::new();
        advertising_failed.start().unwrap();
        advertising_failed.manager_update(ManagerAvailability::PoweredOn);
        advertising_failed.did_add_service(false);
        assert_eq!(
            advertising_failed.did_start_advertising(true),
            PeripheralAction::RemoveService
        );
        assert_eq!(advertising_failed.state(), BluetoothPeripheralState::Failed);
        assert_eq!(
            advertising_failed.error(),
            Some(BluetoothPeripheralError::NativeFailure)
        );
    }

    #[test]
    fn stop_cleans_only_owned_service_and_power_off_requires_republication() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        assert_eq!(
            policy.stop(),
            PeripheralCleanup {
                stop_advertising: true,
                remove_service: true,
            }
        );
        policy.service_removed();
        assert_eq!(policy.state(), BluetoothPeripheralState::Stopped);
        assert_eq!(policy.start(), Ok(()));
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            PeripheralAction::ForgetNativeService
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::WaitingForPower);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
    }

    #[test]
    fn stop_during_advertising_start_defers_service_removal_until_callback() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        assert_eq!(
            policy.did_add_service(false),
            PeripheralAction::StartAdvertising
        );
        assert_eq!(
            policy.stop(),
            PeripheralCleanup {
                stop_advertising: true,
                remove_service: false,
            }
        );
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::None
        );
        assert_eq!(
            policy.did_start_advertising(false),
            PeripheralAction::RemoveService
        );
    }

    #[test]
    fn powered_off_stopped_service_is_forgotten_before_a_new_request() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        policy.stop();
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            PeripheralAction::RemoveService
        );
        assert!(!policy.service_added());
        policy.service_removed();
        policy.start().unwrap();
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
    }

    #[test]
    fn stopped_service_is_removed_after_authorization_recovers() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        policy.manager_update(ManagerAvailability::Unauthorized);
        assert_eq!(
            policy.stop(),
            PeripheralCleanup {
                stop_advertising: false,
                remove_service: true,
            }
        );
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::RemoveService
        );
    }

    #[test]
    fn ambiguous_manager_state_removes_published_service_before_restart() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        assert_eq!(
            policy.manager_update(ManagerAvailability::Other),
            PeripheralAction::None
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::Unavailable);
        assert_eq!(policy.error(), Some(BluetoothPeripheralError::Unavailable));
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::RemoveService
        );
        policy.service_removed();
        policy.start().unwrap();
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
    }

    #[test]
    fn powered_off_active_service_is_republished_after_power_returns() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.did_add_service(false);
        policy.did_start_advertising(false);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            PeripheralAction::ForgetNativeService
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::WaitingForPower);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            PeripheralAction::AddService
        );
    }

    #[test]
    fn interrupted_uncorrelated_native_callback_quarantines_restart() {
        let mut policy = PeripheralPolicy::new();
        policy.start().unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            PeripheralAction::None
        );
        assert_eq!(policy.state(), BluetoothPeripheralState::Unavailable);
        assert_eq!(policy.error(), Some(BluetoothPeripheralError::Unavailable));
        assert_eq!(policy.start(), Err(BluetoothPeripheralError::AlreadyActive));
        assert_eq!(
            policy.did_add_service(false),
            PeripheralAction::RemoveService
        );
        assert_eq!(policy.start(), Ok(()));
    }
}
