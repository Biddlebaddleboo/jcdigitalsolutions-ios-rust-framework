use framework_bluetooth::{
    BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES, BluetoothCharacteristicUuid,
    BluetoothCharacteristicWriteError, BluetoothCharacteristicWriteState, BluetoothConnectionState,
    BluetoothPeripheralId, BluetoothServiceDiscoveryState, BluetoothServiceUuid,
};

use crate::characteristic_read_selection::UniqueMatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WriteDecision {
    Ignore,
    Write,
    Failed {
        peripheral: BluetoothPeripheralId,
        service: BluetoothServiceUuid,
        characteristic: BluetoothCharacteristicUuid,
        error: BluetoothCharacteristicWriteError,
    },
    Completed {
        peripheral: BluetoothPeripheralId,
        service: BluetoothServiceUuid,
        characteristic: BluetoothCharacteristicUuid,
    },
}

pub(crate) struct WriteRequest {
    pub(crate) peripheral: BluetoothPeripheralId,
    pub(crate) service: BluetoothServiceUuid,
    pub(crate) characteristic: BluetoothCharacteristicUuid,
    pub(crate) connection_state: BluetoothConnectionState,
    pub(crate) native_connected: bool,
    pub(crate) manager_supported: bool,
    pub(crate) service_state: BluetoothServiceDiscoveryState,
    pub(crate) service_discovered_for_peer: bool,
    pub(crate) service_match: Result<UniqueMatch, ()>,
    pub(crate) payload_len: usize,
    pub(crate) maximum_write_len: usize,
}

/// One-at-a-time write-with-response lifecycle independent of CoreBluetooth callbacks.
pub(crate) struct CharacteristicWritePolicy {
    peripheral: Option<BluetoothPeripheralId>,
    service: Option<BluetoothServiceUuid>,
    characteristic: Option<BluetoothCharacteristicUuid>,
    state: BluetoothCharacteristicWriteState,
    callback_ambiguous: bool,
}

impl CharacteristicWritePolicy {
    pub(crate) const fn new() -> Self {
        Self {
            peripheral: None,
            service: None,
            characteristic: None,
            state: BluetoothCharacteristicWriteState::Idle,
            callback_ambiguous: false,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothCharacteristicWriteState {
        self.state
    }

    pub(crate) const fn is_active(&self) -> bool {
        matches!(
            self.state,
            BluetoothCharacteristicWriteState::DiscoveringCharacteristics
                | BluetoothCharacteristicWriteState::Writing
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

    pub(crate) const fn callback_ambiguous(&self) -> bool {
        self.callback_ambiguous
    }

    pub(crate) fn begin(
        &mut self,
        request: WriteRequest,
    ) -> Result<usize, BluetoothCharacteristicWriteError> {
        let WriteRequest {
            peripheral,
            service,
            characteristic,
            connection_state,
            native_connected,
            manager_supported,
            service_state,
            service_discovered_for_peer,
            service_match,
            payload_len,
            maximum_write_len,
        } = request;
        if self.callback_ambiguous {
            return Err(BluetoothCharacteristicWriteError::Unavailable);
        }
        if self.is_active() || service_state == BluetoothServiceDiscoveryState::Discovering {
            return Err(BluetoothCharacteristicWriteError::AlreadyActive);
        }
        if connection_state != BluetoothConnectionState::Connected || !native_connected {
            return Err(BluetoothCharacteristicWriteError::NotConnected);
        }
        if !manager_supported {
            return Err(BluetoothCharacteristicWriteError::Unsupported);
        }
        if service_state != BluetoothServiceDiscoveryState::Completed
            || !service_discovered_for_peer
        {
            return Err(BluetoothCharacteristicWriteError::ServiceDiscoveryRequired);
        }
        if payload_len > BLUETOOTH_GATT_ATTRIBUTE_VALUE_MAX_BYTES || payload_len > maximum_write_len
        {
            return Err(BluetoothCharacteristicWriteError::PayloadTooLarge);
        }
        let index = match service_match
            .map_err(|()| BluetoothCharacteristicWriteError::Unavailable)?
        {
            UniqueMatch::Missing => return Err(BluetoothCharacteristicWriteError::ServiceNotFound),
            UniqueMatch::Multiple => {
                return Err(BluetoothCharacteristicWriteError::AmbiguousService);
            }
            UniqueMatch::One(index) => index,
        };
        self.peripheral = Some(peripheral);
        self.service = Some(service);
        self.characteristic = Some(characteristic);
        self.state = BluetoothCharacteristicWriteState::DiscoveringCharacteristics;
        Ok(index)
    }

    pub(crate) fn characteristics_discovered(
        &mut self,
        peripheral: BluetoothPeripheralId,
        had_error: bool,
        selection: Result<(UniqueMatch, bool), ()>,
    ) -> WriteDecision {
        if !self.active_for(
            peripheral,
            BluetoothCharacteristicWriteState::DiscoveringCharacteristics,
        ) {
            return WriteDecision::Ignore;
        }
        if had_error {
            return self.fail(BluetoothCharacteristicWriteError::NativeFailure);
        }
        let (selection, writable) = match selection {
            Ok(selection) => selection,
            Err(()) => return self.fail(BluetoothCharacteristicWriteError::Unavailable),
        };
        match selection {
            UniqueMatch::Missing => {
                self.fail(BluetoothCharacteristicWriteError::CharacteristicNotFound)
            }
            UniqueMatch::Multiple => {
                self.fail(BluetoothCharacteristicWriteError::AmbiguousCharacteristic)
            }
            UniqueMatch::One(_) if !writable => {
                self.fail(BluetoothCharacteristicWriteError::NotWritable)
            }
            UniqueMatch::One(_) => {
                self.state = BluetoothCharacteristicWriteState::Writing;
                WriteDecision::Write
            }
        }
    }

    pub(crate) fn value_written(
        &mut self,
        peripheral: BluetoothPeripheralId,
        characteristic: BluetoothCharacteristicUuid,
        had_error: bool,
    ) -> WriteDecision {
        if !self.active_for(peripheral, BluetoothCharacteristicWriteState::Writing)
            || self.characteristic != Some(characteristic)
        {
            return WriteDecision::Ignore;
        }
        if had_error {
            return self.fail(BluetoothCharacteristicWriteError::NativeFailure);
        }
        let service = self.service.expect("active write has a service");
        self.finish(BluetoothCharacteristicWriteState::Completed);
        WriteDecision::Completed {
            peripheral,
            service,
            characteristic,
        }
    }

    pub(crate) fn disconnected(&mut self, peripheral: BluetoothPeripheralId) -> WriteDecision {
        if self.peripheral == Some(peripheral) && self.is_active() {
            // CoreBluetooth characteristic-discovery and write callbacks have no operation token.
            // A late callback could be mistaken for a later same-peer request, so this backend
            // instance cannot safely issue another write after disconnect interrupts either step.
            self.callback_ambiguous = true;
        }
        self.terminate(peripheral, BluetoothCharacteristicWriteError::Disconnected)
    }

    pub(crate) fn terminate(
        &mut self,
        peripheral: BluetoothPeripheralId,
        error: BluetoothCharacteristicWriteError,
    ) -> WriteDecision {
        if self.peripheral != Some(peripheral) || !self.is_active() {
            return WriteDecision::Ignore;
        }
        self.fail(error)
    }

    pub(crate) fn callback_panicked(&mut self) -> WriteDecision {
        if self.is_active() {
            // The callback may have partially processed an untagged native operation.
            self.callback_ambiguous = true;
        }
        self.fail(BluetoothCharacteristicWriteError::Unavailable)
    }

    pub(crate) fn teardown(&mut self) {
        if self.is_active() {
            self.finish(BluetoothCharacteristicWriteState::Failed);
        }
    }

    fn active_for(
        &self,
        peripheral: BluetoothPeripheralId,
        state: BluetoothCharacteristicWriteState,
    ) -> bool {
        self.peripheral == Some(peripheral) && self.state == state
    }

    fn fail(&mut self, error: BluetoothCharacteristicWriteError) -> WriteDecision {
        let Some(peripheral) = self.peripheral else {
            return WriteDecision::Ignore;
        };
        let Some(service) = self.service else {
            return WriteDecision::Ignore;
        };
        let Some(characteristic) = self.characteristic else {
            return WriteDecision::Ignore;
        };
        self.finish(BluetoothCharacteristicWriteState::Failed);
        WriteDecision::Failed {
            peripheral,
            service,
            characteristic,
            error,
        }
    }

    fn finish(&mut self, state: BluetoothCharacteristicWriteState) {
        self.state = state;
        self.peripheral = None;
        self.service = None;
        self.characteristic = None;
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

    fn failed(error: BluetoothCharacteristicWriteError) -> WriteDecision {
        WriteDecision::Failed {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
            error,
        }
    }

    fn completed() -> WriteDecision {
        WriteDecision::Completed {
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
        payload_len: usize,
        maximum_write_len: usize,
    ) -> WriteRequest {
        WriteRequest {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
            connection_state,
            native_connected,
            manager_supported,
            service_state,
            service_discovered_for_peer: true,
            service_match,
            payload_len,
            maximum_write_len,
        }
    }

    fn begin(policy: &mut CharacteristicWritePolicy) {
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                20,
                20,
            )),
            Ok(0)
        );
    }

    #[test]
    fn begin_rejects_disconnect_unsupported_and_incomplete_peer_scoped_discovery() {
        let mut policy = CharacteristicWritePolicy::new();
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Idle,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::NotConnected)
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                false,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::Unsupported)
        );
        let mut foreign_peer = request(
            BluetoothConnectionState::Connected,
            true,
            true,
            BluetoothServiceDiscoveryState::Completed,
            Ok(UniqueMatch::One(0)),
            1,
            20,
        );
        foreign_peer.service_discovered_for_peer = false;
        assert_eq!(
            policy.begin(foreign_peer),
            Err(BluetoothCharacteristicWriteError::ServiceDiscoveryRequired)
        );
    }

    #[test]
    fn begin_rejects_attribute_and_negotiated_write_size_overflow() {
        let mut policy = CharacteristicWritePolicy::new();
        for (length, max) in [(513, 512), (21, 20)] {
            assert_eq!(
                policy.begin(request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::One(0)),
                    length,
                    max,
                )),
                Err(BluetoothCharacteristicWriteError::PayloadTooLarge)
            );
        }
        for length in [0, 20, 512] {
            let max = if length <= 20 { 20 } else { 512 };
            assert_eq!(
                policy.begin(request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::One(0)),
                    length,
                    max,
                )),
                Ok(0)
            );
            policy.teardown();
        }
    }

    #[test]
    fn begin_rejects_service_absence_ambiguity_overlap_and_active_discovery() {
        let mut policy = CharacteristicWritePolicy::new();
        for (selection, error) in [
            (
                Ok(UniqueMatch::Missing),
                BluetoothCharacteristicWriteError::ServiceNotFound,
            ),
            (
                Ok(UniqueMatch::Multiple),
                BluetoothCharacteristicWriteError::AmbiguousService,
            ),
            (Err(()), BluetoothCharacteristicWriteError::Unavailable),
        ] {
            assert_eq!(
                policy.begin(request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    selection,
                    1,
                    20,
                )),
                Err(error)
            );
        }
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Discovering,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::AlreadyActive)
        );
        begin(&mut policy);
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::AlreadyActive)
        );
    }

    #[test]
    fn characteristic_discovery_requires_one_writable_match() {
        for (selection, error) in [
            (
                Ok((UniqueMatch::Missing, false)),
                BluetoothCharacteristicWriteError::CharacteristicNotFound,
            ),
            (
                Ok((UniqueMatch::Multiple, false)),
                BluetoothCharacteristicWriteError::AmbiguousCharacteristic,
            ),
            (
                Ok((UniqueMatch::One(0), false)),
                BluetoothCharacteristicWriteError::NotWritable,
            ),
            (Err(()), BluetoothCharacteristicWriteError::Unavailable),
        ] {
            let mut policy = CharacteristicWritePolicy::new();
            begin(&mut policy);
            assert_eq!(
                policy.characteristics_discovered(id(), false, selection),
                failed(error)
            );
        }
        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.characteristics_discovered(id(), true, Ok((UniqueMatch::One(0), true))),
            failed(BluetoothCharacteristicWriteError::NativeFailure)
        );
    }

    #[test]
    fn write_completion_failure_disconnect_and_late_callbacks_are_terminal_once() {
        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true))),
            WriteDecision::Write
        );
        assert_eq!(
            policy.value_written(id(), characteristic(), false),
            completed()
        );
        assert_eq!(
            policy.value_written(id(), characteristic(), false),
            WriteDecision::Ignore
        );

        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.value_written(id(), characteristic(), true),
            failed(BluetoothCharacteristicWriteError::NativeFailure)
        );
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.disconnected(id()),
            failed(BluetoothCharacteristicWriteError::Disconnected)
        );
        assert_eq!(policy.callback_panicked(), WriteDecision::Ignore);
    }

    #[test]
    fn interrupted_in_flight_write_quarantines_later_same_peer_operations() {
        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.disconnected(id()),
            failed(BluetoothCharacteristicWriteError::Disconnected)
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::Unavailable)
        );
        assert_eq!(
            policy.value_written(id(), characteristic(), false),
            WriteDecision::Ignore
        );
    }

    #[test]
    fn interrupted_characteristic_discovery_quarantines_late_callback_and_later_writes() {
        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.disconnected(id()),
            failed(BluetoothCharacteristicWriteError::Disconnected)
        );
        assert_eq!(
            policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true))),
            WriteDecision::Ignore
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::Unavailable)
        );
    }

    #[test]
    fn callback_panic_during_discovery_or_write_quarantines_later_operations() {
        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.callback_panicked(),
            failed(BluetoothCharacteristicWriteError::Unavailable)
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::Unavailable)
        );

        let mut policy = CharacteristicWritePolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.callback_panicked(),
            failed(BluetoothCharacteristicWriteError::Unavailable)
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
                1,
                20,
            )),
            Err(BluetoothCharacteristicWriteError::Unavailable)
        );
    }
}
