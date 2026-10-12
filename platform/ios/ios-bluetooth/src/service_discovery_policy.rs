use framework_bluetooth::{
    BluetoothConnectionState, BluetoothPeripheralId, BluetoothServiceDiscoveryError,
    BluetoothServiceDiscoveryEvent, BluetoothServiceDiscoveryState,
};

/// One-at-a-time GATT service-discovery lifecycle, independent of CoreBluetooth callbacks.
pub(crate) struct ServiceDiscoveryPolicy {
    peripheral: Option<BluetoothPeripheralId>,
    state: BluetoothServiceDiscoveryState,
}

impl ServiceDiscoveryPolicy {
    pub(crate) const fn new() -> Self {
        Self {
            peripheral: None,
            state: BluetoothServiceDiscoveryState::Idle,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothServiceDiscoveryState {
        self.state
    }

    pub(crate) const fn peripheral(&self) -> Option<BluetoothPeripheralId> {
        self.peripheral
    }

    pub(crate) fn request(
        &mut self,
        peripheral: BluetoothPeripheralId,
        connection_state: BluetoothConnectionState,
        native_connected: bool,
    ) -> Result<(), BluetoothServiceDiscoveryError> {
        if self.state == BluetoothServiceDiscoveryState::Discovering {
            return Err(BluetoothServiceDiscoveryError::AlreadyActive);
        }
        if connection_state != BluetoothConnectionState::Connected || !native_connected {
            return Err(BluetoothServiceDiscoveryError::NotConnected);
        }
        self.peripheral = Some(peripheral);
        self.state = BluetoothServiceDiscoveryState::Discovering;
        Ok(())
    }

    pub(crate) fn is_active_for(&self, peripheral: BluetoothPeripheralId) -> bool {
        self.peripheral == Some(peripheral)
            && matches!(self.state, BluetoothServiceDiscoveryState::Discovering)
    }

    pub(crate) fn complete(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> Option<BluetoothServiceDiscoveryEvent> {
        if !self.is_active_for(peripheral) {
            return None;
        }
        self.peripheral = None;
        self.state = BluetoothServiceDiscoveryState::Completed;
        Some(BluetoothServiceDiscoveryEvent::Completed { peripheral })
    }

    pub(crate) fn fail(
        &mut self,
        peripheral: BluetoothPeripheralId,
        error: BluetoothServiceDiscoveryError,
    ) -> Option<BluetoothServiceDiscoveryEvent> {
        if !self.is_active_for(peripheral) {
            return None;
        }
        self.peripheral = None;
        self.state = BluetoothServiceDiscoveryState::Failed;
        Some(BluetoothServiceDiscoveryEvent::Failed { peripheral, error })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u8) -> BluetoothPeripheralId {
        BluetoothPeripheralId::from_uuid_bytes([value; 16])
    }

    #[test]
    fn request_rejects_disconnected_and_pending_connections() {
        let mut policy = ServiceDiscoveryPolicy::new();
        for connection_state in [
            BluetoothConnectionState::Idle,
            BluetoothConnectionState::Connecting,
            BluetoothConnectionState::Disconnected,
        ] {
            assert_eq!(
                policy.request(id(1), connection_state, true),
                Err(BluetoothServiceDiscoveryError::NotConnected)
            );
        }
        assert_eq!(
            policy.request(id(1), BluetoothConnectionState::Connected, false),
            Err(BluetoothServiceDiscoveryError::NotConnected)
        );
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Idle);
    }

    #[test]
    fn request_rejects_overlap_until_a_terminal_callback() {
        let mut policy = ServiceDiscoveryPolicy::new();
        assert_eq!(
            policy.request(id(1), BluetoothConnectionState::Connected, true),
            Ok(())
        );
        assert_eq!(
            policy.request(id(1), BluetoothConnectionState::Connected, true),
            Err(BluetoothServiceDiscoveryError::AlreadyActive)
        );
        assert_eq!(
            policy.complete(id(1)),
            Some(BluetoothServiceDiscoveryEvent::Completed { peripheral: id(1) })
        );
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Completed);
        assert_eq!(
            policy.request(id(1), BluetoothConnectionState::Connected, true),
            Ok(())
        );
    }

    #[test]
    fn mismatched_late_callback_cannot_finish_the_selected_request() {
        let mut policy = ServiceDiscoveryPolicy::new();
        policy
            .request(id(1), BluetoothConnectionState::Connected, true)
            .unwrap();
        assert_eq!(policy.complete(id(2)), None);
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Discovering);
        assert_eq!(
            policy.fail(id(1), BluetoothServiceDiscoveryError::Disconnected),
            Some(BluetoothServiceDiscoveryEvent::Failed {
                peripheral: id(1),
                error: BluetoothServiceDiscoveryError::Disconnected,
            })
        );
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Failed);
        assert_eq!(policy.peripheral(), None);
        assert_eq!(policy.complete(id(1)), None);
    }

    #[test]
    fn callback_panic_failure_is_terminal_and_allows_a_later_request() {
        let mut policy = ServiceDiscoveryPolicy::new();
        policy
            .request(id(1), BluetoothConnectionState::Connected, true)
            .unwrap();
        assert_eq!(
            policy.fail(id(1), BluetoothServiceDiscoveryError::Unavailable),
            Some(BluetoothServiceDiscoveryEvent::Failed {
                peripheral: id(1),
                error: BluetoothServiceDiscoveryError::Unavailable,
            })
        );
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Failed);
        assert_eq!(
            policy.request(id(1), BluetoothConnectionState::Connected, true),
            Ok(())
        );
    }

    #[test]
    fn connection_cancellation_fails_discovery_and_ignores_a_late_callback() {
        let mut policy = ServiceDiscoveryPolicy::new();
        policy
            .request(id(1), BluetoothConnectionState::Connected, true)
            .unwrap();
        assert_eq!(
            policy.fail(id(1), BluetoothServiceDiscoveryError::Disconnected),
            Some(BluetoothServiceDiscoveryEvent::Failed {
                peripheral: id(1),
                error: BluetoothServiceDiscoveryError::Disconnected,
            })
        );
        assert_eq!(policy.state(), BluetoothServiceDiscoveryState::Failed);
        assert_eq!(policy.complete(id(1)), None);
    }
}
