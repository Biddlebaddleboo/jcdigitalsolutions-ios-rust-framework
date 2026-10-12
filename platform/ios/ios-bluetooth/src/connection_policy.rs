use framework_bluetooth::{
    BluetoothConnectionError, BluetoothConnectionEvent, BluetoothConnectionState,
    BluetoothPeripheralId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManagerAvailability {
    PoweredOn,
    PoweredOff,
    Unauthorized,
    Unsupported,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConnectionAction {
    None,
    Lookup(BluetoothPeripheralId),
    Connect(BluetoothPeripheralId),
    Cancel(BluetoothPeripheralId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConnectionEffect {
    pub(crate) action: ConnectionAction,
    pub(crate) event: Option<BluetoothConnectionEvent>,
}

impl ConnectionEffect {
    const fn none() -> Self {
        Self {
            action: ConnectionAction::None,
            event: None,
        }
    }
}

/// One-at-a-time connection policy separated from CoreBluetooth for host regression tests.
pub(crate) struct ConnectionPolicy {
    target: Option<BluetoothPeripheralId>,
    state: BluetoothConnectionState,
    native_started: bool,
    lookup_pending: bool,
    cancel_requested: bool,
}

impl ConnectionPolicy {
    pub(crate) const fn new() -> Self {
        Self {
            target: None,
            state: BluetoothConnectionState::Idle,
            native_started: false,
            lookup_pending: false,
            cancel_requested: false,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothConnectionState {
        self.state
    }

    pub(crate) const fn target(&self) -> Option<BluetoothPeripheralId> {
        self.target
    }

    pub(crate) fn request(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> Result<ConnectionEffect, BluetoothConnectionError> {
        if self.target.is_some() {
            return Err(BluetoothConnectionError::AlreadyActive);
        }
        self.target = Some(peripheral);
        self.native_started = false;
        self.lookup_pending = false;
        self.cancel_requested = false;
        Ok(self.transition(BluetoothConnectionState::Starting))
    }

    pub(crate) fn manager_update(&mut self, availability: ManagerAvailability) -> ConnectionEffect {
        let Some(peripheral) = self.target else {
            return ConnectionEffect::none();
        };
        if self.native_started || self.lookup_pending {
            return ConnectionEffect::none();
        }
        match availability {
            ManagerAvailability::PoweredOn => {
                self.lookup_pending = true;
                ConnectionEffect {
                    action: ConnectionAction::Lookup(peripheral),
                    event: None,
                }
            }
            ManagerAvailability::PoweredOff => {
                self.transition(BluetoothConnectionState::WaitingForPower)
            }
            ManagerAvailability::Unauthorized => self.fail(BluetoothConnectionError::Unauthorized),
            ManagerAvailability::Unsupported => self.fail(BluetoothConnectionError::Unsupported),
            ManagerAvailability::Other => self.transition(BluetoothConnectionState::Starting),
        }
    }

    pub(crate) fn lookup_result(
        &mut self,
        peripheral: BluetoothPeripheralId,
        found: bool,
    ) -> ConnectionEffect {
        if self.target != Some(peripheral) || !self.lookup_pending {
            return ConnectionEffect::none();
        }
        self.lookup_pending = false;
        if !found {
            return self.fail(BluetoothConnectionError::PeripheralUnavailable);
        }
        self.native_started = true;
        ConnectionEffect {
            action: ConnectionAction::Connect(peripheral),
            event: self.transition(BluetoothConnectionState::Connecting).event,
        }
    }

    pub(crate) fn cancel(&mut self) -> ConnectionEffect {
        let Some(peripheral) = self.target else {
            return ConnectionEffect::none();
        };
        if self.native_started {
            if self.cancel_requested {
                return ConnectionEffect::none();
            }
            self.cancel_requested = true;
            let event = self
                .transition(BluetoothConnectionState::Disconnecting)
                .event;
            ConnectionEffect {
                action: ConnectionAction::Cancel(peripheral),
                event,
            }
        } else {
            self.terminal(BluetoothConnectionState::Disconnected);
            ConnectionEffect {
                action: ConnectionAction::None,
                event: Some(BluetoothConnectionEvent::StateChanged {
                    peripheral,
                    state: BluetoothConnectionState::Disconnected,
                }),
            }
        }
    }

    pub(crate) fn did_connect(&mut self, peripheral: BluetoothPeripheralId) -> ConnectionEffect {
        if self.target != Some(peripheral) || !self.native_started {
            return ConnectionEffect::none();
        }
        if self.cancel_requested {
            return ConnectionEffect {
                action: ConnectionAction::Cancel(peripheral),
                event: None,
            };
        }
        self.transition(BluetoothConnectionState::Connected)
    }

    pub(crate) fn did_fail_to_connect(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> ConnectionEffect {
        if self.target != Some(peripheral) || !self.native_started {
            return ConnectionEffect::none();
        }
        if self.cancel_requested {
            return self.finish_disconnected(peripheral);
        }
        self.fail(BluetoothConnectionError::ConnectionFailed)
    }

    pub(crate) fn did_disconnect(
        &mut self,
        peripheral: BluetoothPeripheralId,
        had_error: bool,
    ) -> ConnectionEffect {
        if self.target != Some(peripheral) || !self.native_started {
            return ConnectionEffect::none();
        }
        if self.cancel_requested || !had_error {
            self.finish_disconnected(peripheral)
        } else {
            self.fail(BluetoothConnectionError::ConnectionFailed)
        }
    }

    pub(crate) fn callback_panicked(&mut self) -> ConnectionEffect {
        let Some(peripheral) = self.target else {
            return ConnectionEffect::none();
        };
        if self.native_started {
            self.cancel_requested = true;
            self.state = BluetoothConnectionState::Disconnecting;
            ConnectionEffect {
                action: ConnectionAction::Cancel(peripheral),
                event: Some(BluetoothConnectionEvent::Failed {
                    peripheral,
                    error: BluetoothConnectionError::Unavailable,
                }),
            }
        } else {
            self.fail(BluetoothConnectionError::Unavailable)
        }
    }

    pub(crate) fn teardown(&mut self) -> ConnectionAction {
        let action = match (self.target, self.native_started) {
            (Some(peripheral), true) => ConnectionAction::Cancel(peripheral),
            _ => ConnectionAction::None,
        };
        self.terminal(BluetoothConnectionState::Disconnected);
        action
    }

    pub(crate) fn native_unavailable(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> ConnectionEffect {
        if self.target != Some(peripheral) {
            return ConnectionEffect::none();
        }
        self.fail(BluetoothConnectionError::Unavailable)
    }

    fn transition(&mut self, state: BluetoothConnectionState) -> ConnectionEffect {
        if self.state == state {
            return ConnectionEffect::none();
        }
        self.state = state;
        ConnectionEffect {
            action: ConnectionAction::None,
            event: self
                .target
                .map(|peripheral| BluetoothConnectionEvent::StateChanged { peripheral, state }),
        }
    }

    fn fail(&mut self, error: BluetoothConnectionError) -> ConnectionEffect {
        let Some(peripheral) = self.target else {
            return ConnectionEffect::none();
        };
        self.terminal(BluetoothConnectionState::Failed);
        ConnectionEffect {
            action: ConnectionAction::None,
            event: Some(BluetoothConnectionEvent::Failed { peripheral, error }),
        }
    }

    fn finish_disconnected(&mut self, peripheral: BluetoothPeripheralId) -> ConnectionEffect {
        self.terminal(BluetoothConnectionState::Disconnected);
        ConnectionEffect {
            action: ConnectionAction::None,
            event: Some(BluetoothConnectionEvent::StateChanged {
                peripheral,
                state: BluetoothConnectionState::Disconnected,
            }),
        }
    }

    fn terminal(&mut self, state: BluetoothConnectionState) {
        self.target = None;
        self.state = state;
        self.native_started = false;
        self.lookup_pending = false;
        self.cancel_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u8) -> BluetoothPeripheralId {
        BluetoothPeripheralId::from_uuid_bytes([value; 16])
    }

    fn event(state: BluetoothConnectionState) -> BluetoothConnectionEvent {
        BluetoothConnectionEvent::StateChanged {
            peripheral: id(1),
            state,
        }
    }

    #[test]
    fn request_waits_for_power_and_retrieval_before_connection() {
        let mut policy = ConnectionPolicy::new();
        let requested = policy.request(id(1)).unwrap();
        assert_eq!(
            requested.event,
            Some(event(BluetoothConnectionState::Starting))
        );
        assert_eq!(requested.action, ConnectionAction::None);
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOff),
            ConnectionEffect {
                action: ConnectionAction::None,
                event: Some(event(BluetoothConnectionState::WaitingForPower)),
            }
        );
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn).action,
            ConnectionAction::Lookup(id(1))
        );
        assert_eq!(
            policy.lookup_result(id(1), true),
            ConnectionEffect {
                action: ConnectionAction::Connect(id(1)),
                event: Some(event(BluetoothConnectionState::Connecting)),
            }
        );
        assert_eq!(
            policy.did_connect(id(1)).event,
            Some(event(BluetoothConnectionState::Connected))
        );
        assert_eq!(policy.state(), BluetoothConnectionState::Connected);
    }

    #[test]
    fn request_rejects_overlap_until_cancel_callback_then_ignores_late_success() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.lookup_result(id(1), true);
        assert_eq!(
            policy.request(id(2)),
            Err(BluetoothConnectionError::AlreadyActive)
        );
        let cancelled = policy.cancel();
        assert_eq!(cancelled.action, ConnectionAction::Cancel(id(1)));
        assert_eq!(
            cancelled.event,
            Some(event(BluetoothConnectionState::Disconnecting))
        );
        assert_eq!(policy.cancel(), ConnectionEffect::none());
        let late_success = policy.did_connect(id(1));
        assert_eq!(late_success.action, ConnectionAction::Cancel(id(1)));
        assert_eq!(late_success.event, None);
        assert_eq!(policy.state(), BluetoothConnectionState::Disconnecting);
        assert_eq!(
            policy.did_disconnect(id(1), false).event,
            Some(event(BluetoothConnectionState::Disconnected))
        );
        assert_eq!(
            policy.request(id(2)).unwrap().event,
            Some(BluetoothConnectionEvent::StateChanged {
                peripheral: id(2),
                state: BluetoothConnectionState::Starting,
            })
        );
    }

    #[test]
    fn cancel_before_native_lookup_ends_request_and_ignores_late_readiness() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        assert_eq!(
            policy.cancel().event,
            Some(event(BluetoothConnectionState::Disconnected))
        );
        assert_eq!(
            policy.manager_update(ManagerAvailability::PoweredOn),
            ConnectionEffect::none()
        );
        assert_eq!(policy.lookup_result(id(1), true), ConnectionEffect::none());
        assert_eq!(policy.state(), BluetoothConnectionState::Disconnected);
    }

    #[test]
    fn unavailable_peer_and_authorization_failures_are_terminal_and_retryable() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        assert_eq!(
            policy.lookup_result(id(1), false).event,
            Some(BluetoothConnectionEvent::Failed {
                peripheral: id(1),
                error: BluetoothConnectionError::PeripheralUnavailable,
            })
        );
        assert_eq!(policy.state(), BluetoothConnectionState::Failed);
        policy.request(id(2)).unwrap();
        assert_eq!(
            policy
                .manager_update(ManagerAvailability::Unauthorized)
                .event,
            Some(BluetoothConnectionEvent::Failed {
                peripheral: id(2),
                error: BluetoothConnectionError::Unauthorized,
            })
        );
        assert_eq!(policy.target(), None);
    }

    #[test]
    fn unsupported_manager_is_a_typed_terminal_failure() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        assert_eq!(
            policy
                .manager_update(ManagerAvailability::Unsupported)
                .event,
            Some(BluetoothConnectionEvent::Failed {
                peripheral: id(1),
                error: BluetoothConnectionError::Unsupported,
            })
        );
        assert_eq!(policy.state(), BluetoothConnectionState::Failed);
    }

    #[test]
    fn mismatched_and_late_native_callbacks_do_not_change_state() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.lookup_result(id(1), true);
        assert_eq!(policy.did_connect(id(2)), ConnectionEffect::none());
        policy.did_fail_to_connect(id(1));
        assert_eq!(policy.state(), BluetoothConnectionState::Failed);
        assert_eq!(policy.did_connect(id(1)), ConnectionEffect::none());
        assert_eq!(policy.did_disconnect(id(1), true), ConnectionEffect::none());
    }

    #[test]
    fn callback_panic_reports_unavailable_and_cancels_native_connection() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.lookup_result(id(1), true);
        let effect = policy.callback_panicked();
        assert_eq!(effect.action, ConnectionAction::Cancel(id(1)));
        assert_eq!(
            effect.event,
            Some(BluetoothConnectionEvent::Failed {
                peripheral: id(1),
                error: BluetoothConnectionError::Unavailable,
            })
        );
        assert_eq!(policy.state(), BluetoothConnectionState::Disconnecting);
        assert_eq!(
            policy.did_disconnect(id(1), false).event,
            Some(event(BluetoothConnectionState::Disconnected))
        );
    }

    #[test]
    fn teardown_cancels_only_a_native_request_and_ignores_late_callbacks() {
        let mut policy = ConnectionPolicy::new();
        policy.request(id(1)).unwrap();
        assert_eq!(policy.teardown(), ConnectionAction::None);
        assert_eq!(policy.state(), BluetoothConnectionState::Disconnected);
        assert_eq!(policy.target(), None);

        policy.request(id(2)).unwrap();
        policy.manager_update(ManagerAvailability::PoweredOn);
        policy.lookup_result(id(2), true);
        assert_eq!(policy.teardown(), ConnectionAction::Cancel(id(2)));
        assert_eq!(policy.state(), BluetoothConnectionState::Disconnected);
        assert_eq!(
            policy.did_disconnect(id(2), false),
            ConnectionEffect::none()
        );
    }
}
