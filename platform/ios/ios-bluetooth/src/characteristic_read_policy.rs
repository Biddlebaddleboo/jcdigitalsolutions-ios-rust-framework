use framework_bluetooth::{
    BluetoothCharacteristicReadError, BluetoothCharacteristicReadState,
    BluetoothCharacteristicUuid, BluetoothConnectionState, BluetoothPeripheralId,
    BluetoothServiceDiscoveryState, BluetoothServiceUuid,
};

use crate::characteristic_read_selection::UniqueMatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadDecision {
    Ignore,
    Read,
    Failed {
        peripheral: BluetoothPeripheralId,
        service: BluetoothServiceUuid,
        characteristic: BluetoothCharacteristicUuid,
        error: BluetoothCharacteristicReadError,
    },
    Completed {
        peripheral: BluetoothPeripheralId,
        service: BluetoothServiceUuid,
        characteristic: BluetoothCharacteristicUuid,
    },
}

pub(crate) struct ReadRequest {
    pub(crate) peripheral: BluetoothPeripheralId,
    pub(crate) service: BluetoothServiceUuid,
    pub(crate) characteristic: BluetoothCharacteristicUuid,
    pub(crate) connection_state: BluetoothConnectionState,
    pub(crate) native_connected: bool,
    pub(crate) manager_supported: bool,
    pub(crate) service_state: BluetoothServiceDiscoveryState,
    pub(crate) service_discovered_for_peer: bool,
    pub(crate) service_match: Result<UniqueMatch, ()>,
}

/// One-at-a-time read lifecycle independent of CoreBluetooth callbacks.
pub(crate) struct CharacteristicReadPolicy {
    peripheral: Option<BluetoothPeripheralId>,
    service: Option<BluetoothServiceUuid>,
    characteristic: Option<BluetoothCharacteristicUuid>,
    state: BluetoothCharacteristicReadState,
}

impl CharacteristicReadPolicy {
    pub(crate) const fn new() -> Self {
        Self {
            peripheral: None,
            service: None,
            characteristic: None,
            state: BluetoothCharacteristicReadState::Idle,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothCharacteristicReadState {
        self.state
    }

    pub(crate) const fn is_active(&self) -> bool {
        matches!(
            self.state,
            BluetoothCharacteristicReadState::DiscoveringCharacteristics
                | BluetoothCharacteristicReadState::Reading
        )
    }

    pub(crate) const fn peripheral(&self) -> Option<BluetoothPeripheralId> {
        self.peripheral
    }

    pub(crate) const fn service(&self) -> Option<BluetoothServiceUuid> {
        self.service
    }

    pub(crate) const fn characteristic(&self) -> Option<BluetoothCharacteristicUuid> {
        self.characteristic
    }

    pub(crate) fn begin(
        &mut self,
        request: ReadRequest,
    ) -> Result<usize, BluetoothCharacteristicReadError> {
        let ReadRequest {
            peripheral,
            service,
            characteristic,
            connection_state,
            native_connected,
            manager_supported,
            service_state,
            service_discovered_for_peer,
            service_match,
        } = request;
        if self.is_active() || service_state == BluetoothServiceDiscoveryState::Discovering {
            return Err(BluetoothCharacteristicReadError::AlreadyActive);
        }
        if connection_state != BluetoothConnectionState::Connected || !native_connected {
            return Err(BluetoothCharacteristicReadError::NotConnected);
        }
        if !manager_supported {
            return Err(BluetoothCharacteristicReadError::Unsupported);
        }
        if service_state != BluetoothServiceDiscoveryState::Completed
            || !service_discovered_for_peer
        {
            return Err(BluetoothCharacteristicReadError::ServiceDiscoveryRequired);
        }
        let index = match service_match
            .map_err(|()| BluetoothCharacteristicReadError::Unavailable)?
        {
            UniqueMatch::Missing => return Err(BluetoothCharacteristicReadError::ServiceNotFound),
            UniqueMatch::Multiple => {
                return Err(BluetoothCharacteristicReadError::AmbiguousService);
            }
            UniqueMatch::One(index) => index,
        };
        self.peripheral = Some(peripheral);
        self.service = Some(service);
        self.characteristic = Some(characteristic);
        self.state = BluetoothCharacteristicReadState::DiscoveringCharacteristics;
        Ok(index)
    }

    pub(crate) fn characteristics_discovered(
        &mut self,
        peripheral: BluetoothPeripheralId,
        had_error: bool,
        selection: Result<(UniqueMatch, bool), ()>,
    ) -> ReadDecision {
        if !self.active_for(
            peripheral,
            BluetoothCharacteristicReadState::DiscoveringCharacteristics,
        ) {
            return ReadDecision::Ignore;
        }
        if had_error {
            return self.fail(BluetoothCharacteristicReadError::NativeFailure);
        }
        let (selection, readable) = match selection {
            Ok(selection) => selection,
            Err(()) => return self.fail(BluetoothCharacteristicReadError::Unavailable),
        };
        match selection {
            UniqueMatch::Missing => {
                self.fail(BluetoothCharacteristicReadError::CharacteristicNotFound)
            }
            UniqueMatch::Multiple => {
                self.fail(BluetoothCharacteristicReadError::AmbiguousCharacteristic)
            }
            UniqueMatch::One(_) if !readable => {
                self.fail(BluetoothCharacteristicReadError::NotReadable)
            }
            UniqueMatch::One(_) => {
                self.state = BluetoothCharacteristicReadState::Reading;
                ReadDecision::Read
            }
        }
    }

    pub(crate) fn value_updated(
        &mut self,
        peripheral: BluetoothPeripheralId,
        characteristic: BluetoothCharacteristicUuid,
        had_error: bool,
        value_error: Option<BluetoothCharacteristicReadError>,
    ) -> ReadDecision {
        if !self.active_for(peripheral, BluetoothCharacteristicReadState::Reading)
            || self.characteristic != Some(characteristic)
        {
            return ReadDecision::Ignore;
        }
        if had_error {
            return self.fail(BluetoothCharacteristicReadError::NativeFailure);
        }
        if let Some(error) = value_error {
            return self.fail(error);
        }
        let service = self.service.expect("active read has a service");
        self.finish(BluetoothCharacteristicReadState::Completed);
        ReadDecision::Completed {
            peripheral,
            service,
            characteristic,
        }
    }

    pub(crate) fn disconnected(&mut self, peripheral: BluetoothPeripheralId) -> ReadDecision {
        self.terminate(peripheral, BluetoothCharacteristicReadError::Disconnected)
    }

    pub(crate) fn terminate(
        &mut self,
        peripheral: BluetoothPeripheralId,
        error: BluetoothCharacteristicReadError,
    ) -> ReadDecision {
        if self.peripheral != Some(peripheral) || !self.is_active() {
            return ReadDecision::Ignore;
        }
        self.fail(error)
    }

    pub(crate) fn callback_panicked(&mut self) -> ReadDecision {
        if !self.is_active() {
            return ReadDecision::Ignore;
        }
        self.fail(BluetoothCharacteristicReadError::Unavailable)
    }

    pub(crate) fn teardown(&mut self) {
        if self.is_active() {
            self.finish(BluetoothCharacteristicReadState::Failed);
        }
    }

    fn active_for(
        &self,
        peripheral: BluetoothPeripheralId,
        state: BluetoothCharacteristicReadState,
    ) -> bool {
        self.peripheral == Some(peripheral) && self.state == state
    }

    fn fail(&mut self, error: BluetoothCharacteristicReadError) -> ReadDecision {
        let Some(peripheral) = self.peripheral else {
            return ReadDecision::Ignore;
        };
        let service = self.service.expect("active read has a service");
        let characteristic = self
            .characteristic
            .expect("active read has a characteristic");
        self.finish(BluetoothCharacteristicReadState::Failed);
        ReadDecision::Failed {
            peripheral,
            service,
            characteristic,
            error,
        }
    }

    fn finish(&mut self, state: BluetoothCharacteristicReadState) {
        self.peripheral = None;
        self.service = None;
        self.characteristic = None;
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> BluetoothPeripheralId {
        BluetoothPeripheralId::from_uuid_bytes([0xA5; 16])
    }

    fn service() -> BluetoothServiceUuid {
        BluetoothServiceUuid::from_uuid_bytes([0x18; 16])
    }

    fn characteristic() -> BluetoothCharacteristicUuid {
        BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16])
    }

    fn failed(error: BluetoothCharacteristicReadError) -> ReadDecision {
        ReadDecision::Failed {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
            error,
        }
    }

    fn completed() -> ReadDecision {
        ReadDecision::Completed {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
        }
    }

    fn request(
        connection_state: BluetoothConnectionState,
        native_connected: bool,
        manager_supported: bool,
        service_state: BluetoothServiceDiscoveryState,
        service_match: Result<UniqueMatch, ()>,
    ) -> ReadRequest {
        ReadRequest {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
            connection_state,
            native_connected,
            manager_supported,
            service_state,
            service_discovered_for_peer: true,
            service_match,
        }
    }

    fn begin(policy: &mut CharacteristicReadPolicy) {
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Ok(0)
        );
    }

    #[test]
    fn begin_rejects_not_connected_unsupported_incomplete_and_overlapping_requests() {
        let mut policy = CharacteristicReadPolicy::new();
        for state in [
            BluetoothConnectionState::Idle,
            BluetoothConnectionState::Connecting,
            BluetoothConnectionState::Disconnected,
        ] {
            assert_eq!(
                policy.begin(request(
                    state,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::One(0)),
                )),
                Err(BluetoothCharacteristicReadError::NotConnected)
            );
        }
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                false,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicReadError::NotConnected)
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                false,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicReadError::Unsupported)
        );
        for state in [
            BluetoothServiceDiscoveryState::Idle,
            BluetoothServiceDiscoveryState::Failed,
        ] {
            assert_eq!(
                policy.begin(request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    state,
                    Ok(UniqueMatch::One(0)),
                )),
                Err(BluetoothCharacteristicReadError::ServiceDiscoveryRequired)
            );
        }
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Discovering,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicReadError::AlreadyActive)
        );
        begin(&mut policy);
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicReadError::AlreadyActive)
        );
    }

    #[test]
    fn begin_rejects_completed_service_state_from_another_peer() {
        let mut policy = CharacteristicReadPolicy::new();
        let mut request = request(
            BluetoothConnectionState::Connected,
            true,
            true,
            BluetoothServiceDiscoveryState::Completed,
            Ok(UniqueMatch::One(0)),
        );
        request.service_discovered_for_peer = false;
        assert_eq!(
            policy.begin(request),
            Err(BluetoothCharacteristicReadError::ServiceDiscoveryRequired)
        );
    }

    #[test]
    fn begin_rejects_absent_duplicate_and_unconvertible_services() {
        let mut policy = CharacteristicReadPolicy::new();
        for (selection, error) in [
            (
                Ok(UniqueMatch::Missing),
                BluetoothCharacteristicReadError::ServiceNotFound,
            ),
            (
                Ok(UniqueMatch::Multiple),
                BluetoothCharacteristicReadError::AmbiguousService,
            ),
            (Err(()), BluetoothCharacteristicReadError::Unavailable),
        ] {
            assert_eq!(
                policy.begin(request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    selection,
                )),
                Err(error)
            );
            assert_eq!(policy.state(), BluetoothCharacteristicReadState::Idle);
        }
    }

    #[test]
    fn characteristic_discovery_rejects_missing_duplicate_unreadable_and_native_failure() {
        for (selection, error) in [
            (
                Ok((UniqueMatch::Missing, false)),
                BluetoothCharacteristicReadError::CharacteristicNotFound,
            ),
            (
                Ok((UniqueMatch::Multiple, false)),
                BluetoothCharacteristicReadError::AmbiguousCharacteristic,
            ),
            (
                Ok((UniqueMatch::One(0), false)),
                BluetoothCharacteristicReadError::NotReadable,
            ),
            (Err(()), BluetoothCharacteristicReadError::Unavailable),
        ] {
            let mut policy = CharacteristicReadPolicy::new();
            begin(&mut policy);
            assert_eq!(
                policy.characteristics_discovered(id(), false, selection),
                failed(error)
            );
            assert_eq!(policy.state(), BluetoothCharacteristicReadState::Failed);
        }
        let mut policy = CharacteristicReadPolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.characteristics_discovered(id(), true, Ok((UniqueMatch::One(0), true))),
            failed(BluetoothCharacteristicReadError::NativeFailure)
        );
    }

    #[test]
    fn read_completion_accepts_empty_and_full_values_and_rejects_oversize() {
        for bytes in [&[][..], &[0x5A; 512][..]] {
            let copied = framework_bluetooth::BluetoothCharacteristicValue::from_bytes(bytes)
                .expect("bounded values copy without truncation");
            assert_eq!(copied.as_bytes(), bytes);
            let mut policy = CharacteristicReadPolicy::new();
            begin(&mut policy);
            assert_eq!(
                policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true))),
                ReadDecision::Read
            );
            assert_eq!(
                policy.value_updated(id(), characteristic(), false, None),
                completed()
            );
        }
        let mut policy = CharacteristicReadPolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert!(framework_bluetooth::BluetoothCharacteristicValue::from_bytes(&[0; 513]).is_none());
        assert_eq!(
            policy.value_updated(
                id(),
                characteristic(),
                false,
                Some(BluetoothCharacteristicReadError::ValueTooLarge)
            ),
            failed(BluetoothCharacteristicReadError::ValueTooLarge)
        );
    }

    #[test]
    fn read_native_error_disconnect_panic_and_late_callbacks_are_terminal_once() {
        let mut policy = CharacteristicReadPolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.value_updated(
                id(),
                characteristic(),
                true,
                Some(BluetoothCharacteristicReadError::Unavailable)
            ),
            failed(BluetoothCharacteristicReadError::NativeFailure)
        );
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.value_updated(
                id(),
                characteristic(),
                false,
                Some(BluetoothCharacteristicReadError::Unavailable)
            ),
            failed(BluetoothCharacteristicReadError::Unavailable)
        );
        assert_eq!(
            policy.value_updated(id(), characteristic(), false, None),
            ReadDecision::Ignore
        );
        begin(&mut policy);
        assert_eq!(
            policy.disconnected(id()),
            failed(BluetoothCharacteristicReadError::Disconnected)
        );
        begin(&mut policy);
        assert_eq!(
            policy.callback_panicked(),
            failed(BluetoothCharacteristicReadError::Unavailable)
        );
    }

    #[test]
    fn mismatched_and_teardown_callbacks_cannot_complete_an_active_read() {
        let mut policy = CharacteristicReadPolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.value_updated(
                BluetoothPeripheralId::from_uuid_bytes([0x44; 16]),
                characteristic(),
                false,
                None
            ),
            ReadDecision::Ignore
        );
        assert_eq!(
            policy.value_updated(
                id(),
                BluetoothCharacteristicUuid::from_uuid_bytes([0x44; 16]),
                false,
                None
            ),
            ReadDecision::Ignore
        );
        policy.teardown();
        assert_eq!(policy.state(), BluetoothCharacteristicReadState::Failed);
        assert_eq!(
            policy.value_updated(id(), characteristic(), false, None),
            ReadDecision::Ignore
        );
    }
}
