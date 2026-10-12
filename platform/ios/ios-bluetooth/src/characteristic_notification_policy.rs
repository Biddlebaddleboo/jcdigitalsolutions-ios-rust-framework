use framework_bluetooth::{
    BluetoothCharacteristicNotificationError, BluetoothCharacteristicNotificationState,
    BluetoothCharacteristicUuid, BluetoothConnectionState, BluetoothPeripheralId,
    BluetoothServiceDiscoveryState, BluetoothServiceUuid,
};

use crate::characteristic_read_selection::UniqueMatch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Disabled,
    Discovering,
    Enabling,
    Enabled,
    Disabling,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NotificationDecision {
    Ignore,
    Enable(usize),
    Enabled,
    Disabled,
    Event {
        peripheral: BluetoothPeripheralId,
        service: BluetoothServiceUuid,
        characteristic: BluetoothCharacteristicUuid,
        error: BluetoothCharacteristicNotificationError,
    },
}

pub(crate) struct NotificationRequest {
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

/// One connected-peer notification/indication subscription.
pub(crate) struct CharacteristicNotificationPolicy {
    peripheral: Option<BluetoothPeripheralId>,
    service: Option<BluetoothServiceUuid>,
    characteristic: Option<BluetoothCharacteristicUuid>,
    phase: Phase,
    callback_ambiguous: bool,
    last_error: Option<BluetoothCharacteristicNotificationError>,
}

impl CharacteristicNotificationPolicy {
    pub(crate) const fn new() -> Self {
        Self {
            peripheral: None,
            service: None,
            characteristic: None,
            phase: Phase::Disabled,
            callback_ambiguous: false,
            last_error: None,
        }
    }

    pub(crate) const fn state(&self) -> BluetoothCharacteristicNotificationState {
        match self.phase {
            Phase::Disabled => BluetoothCharacteristicNotificationState::Disabled,
            Phase::Discovering | Phase::Enabling => {
                BluetoothCharacteristicNotificationState::Enabling
            }
            Phase::Enabled => BluetoothCharacteristicNotificationState::Enabled,
            Phase::Disabling => BluetoothCharacteristicNotificationState::Disabling,
            Phase::Failed => BluetoothCharacteristicNotificationState::Failed,
        }
    }

    pub(crate) const fn is_active(&self) -> bool {
        matches!(
            self.phase,
            Phase::Discovering | Phase::Enabling | Phase::Enabled | Phase::Disabling
        )
    }

    pub(crate) const fn is_transitioning(&self) -> bool {
        matches!(
            self.phase,
            Phase::Discovering | Phase::Enabling | Phase::Disabling
        )
    }

    pub(crate) const fn is_discovering(&self) -> bool {
        matches!(self.phase, Phase::Discovering)
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

    pub(crate) const fn last_error(&self) -> Option<BluetoothCharacteristicNotificationError> {
        self.last_error
    }

    pub(crate) const fn callback_ambiguous(&self) -> bool {
        self.callback_ambiguous
    }

    pub(crate) fn begin(
        &mut self,
        request: NotificationRequest,
    ) -> Result<usize, BluetoothCharacteristicNotificationError> {
        let NotificationRequest {
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
        if self.callback_ambiguous {
            return Err(BluetoothCharacteristicNotificationError::Unavailable);
        }
        if self.is_active() || service_state == BluetoothServiceDiscoveryState::Discovering {
            return Err(BluetoothCharacteristicNotificationError::AlreadyActive);
        }
        if connection_state != BluetoothConnectionState::Connected || !native_connected {
            return Err(BluetoothCharacteristicNotificationError::NotConnected);
        }
        if !manager_supported {
            return Err(BluetoothCharacteristicNotificationError::Unsupported);
        }
        if service_state != BluetoothServiceDiscoveryState::Completed
            || !service_discovered_for_peer
        {
            return Err(BluetoothCharacteristicNotificationError::ServiceDiscoveryRequired);
        }
        let index = match service_match
            .map_err(|()| BluetoothCharacteristicNotificationError::Unavailable)?
        {
            UniqueMatch::Missing => {
                return Err(BluetoothCharacteristicNotificationError::ServiceNotFound);
            }
            UniqueMatch::Multiple => {
                return Err(BluetoothCharacteristicNotificationError::AmbiguousService);
            }
            UniqueMatch::One(index) => index,
        };
        self.peripheral = Some(peripheral);
        self.service = Some(service);
        self.characteristic = Some(characteristic);
        self.phase = Phase::Discovering;
        self.last_error = None;
        Ok(index)
    }

    pub(crate) fn characteristics_discovered(
        &mut self,
        peripheral: BluetoothPeripheralId,
        had_error: bool,
        selection: Result<(UniqueMatch, bool), ()>,
    ) -> NotificationDecision {
        if !self.active_for(peripheral, Phase::Discovering) {
            return NotificationDecision::Ignore;
        }
        if had_error {
            return self.fail(
                BluetoothCharacteristicNotificationError::NativeFailure,
                false,
            );
        }
        let (selection, notifiable) = match selection {
            Ok(selection) => selection,
            Err(()) => {
                return self.fail(BluetoothCharacteristicNotificationError::Unavailable, false);
            }
        };
        match selection {
            UniqueMatch::Missing => self.fail(
                BluetoothCharacteristicNotificationError::CharacteristicNotFound,
                false,
            ),
            UniqueMatch::Multiple => self.fail(
                BluetoothCharacteristicNotificationError::AmbiguousCharacteristic,
                false,
            ),
            UniqueMatch::One(_) if !notifiable => self.fail(
                BluetoothCharacteristicNotificationError::NotNotifiable,
                false,
            ),
            UniqueMatch::One(index) => {
                self.phase = Phase::Enabling;
                NotificationDecision::Enable(index)
            }
        }
    }

    pub(crate) fn begin_disable(
        &mut self,
    ) -> Result<
        (
            BluetoothPeripheralId,
            BluetoothServiceUuid,
            BluetoothCharacteristicUuid,
        ),
        BluetoothCharacteristicNotificationError,
    > {
        if self.phase == Phase::Disabling
            || self.phase == Phase::Enabling
            || self.phase == Phase::Discovering
        {
            return Err(BluetoothCharacteristicNotificationError::AlreadyActive);
        }
        if self.phase != Phase::Enabled {
            return Err(BluetoothCharacteristicNotificationError::NotSubscribed);
        }
        let peripheral = self
            .peripheral
            .ok_or(BluetoothCharacteristicNotificationError::Unavailable)?;
        let service = self
            .service
            .ok_or(BluetoothCharacteristicNotificationError::Unavailable)?;
        let characteristic = self
            .characteristic
            .ok_or(BluetoothCharacteristicNotificationError::Unavailable)?;
        self.phase = Phase::Disabling;
        Ok((peripheral, service, characteristic))
    }

    pub(crate) fn notification_state_changed(
        &mut self,
        peripheral: BluetoothPeripheralId,
        characteristic: BluetoothCharacteristicUuid,
        is_notifying: bool,
        had_error: bool,
    ) -> NotificationDecision {
        if self.peripheral != Some(peripheral) || self.characteristic != Some(characteristic) {
            return NotificationDecision::Ignore;
        }
        match self.phase {
            Phase::Enabling if !had_error && is_notifying => {
                self.phase = Phase::Enabled;
                self.last_error = None;
                NotificationDecision::Enabled
            }
            Phase::Disabling if !had_error && !is_notifying => {
                // CoreBluetooth value callbacks carry no operation token. Keep the native
                // characteristic as a tombstone and reject later reads/subscriptions rather
                // than misattribute a queued notification value after disable confirmation.
                self.callback_ambiguous = true;
                self.finish(Phase::Disabled);
                NotificationDecision::Disabled
            }
            Phase::Enabling if had_error && is_notifying => {
                self.phase = Phase::Enabled;
                self.failure(BluetoothCharacteristicNotificationError::NativeFailure)
            }
            Phase::Disabling if had_error && is_notifying => {
                self.phase = Phase::Enabled;
                self.failure(BluetoothCharacteristicNotificationError::NativeFailure)
            }
            Phase::Disabling if had_error && !is_notifying => {
                self.report_and_finish(BluetoothCharacteristicNotificationError::NativeFailure)
            }
            Phase::Enabling if had_error => self.fail(
                BluetoothCharacteristicNotificationError::NativeFailure,
                false,
            ),
            Phase::Enabling | Phase::Disabling => {
                self.fail(BluetoothCharacteristicNotificationError::Unavailable, true)
            }
            _ => NotificationDecision::Ignore,
        }
    }

    pub(crate) fn value_updated(
        &mut self,
        peripheral: BluetoothPeripheralId,
        characteristic: BluetoothCharacteristicUuid,
        error: Option<BluetoothCharacteristicNotificationError>,
    ) -> Option<BluetoothServiceUuid> {
        if self.peripheral != Some(peripheral)
            || self.characteristic != Some(characteristic)
            || !matches!(self.phase, Phase::Enabled | Phase::Disabling)
        {
            return None;
        }
        self.last_error = error;
        self.service
    }

    pub(crate) fn disconnected(
        &mut self,
        peripheral: BluetoothPeripheralId,
    ) -> NotificationDecision {
        if self.peripheral == Some(peripheral) && self.is_active() {
            // CoreBluetooth does not tag notification-state/value callbacks with a subscription
            // generation. Recreating the backend is the safe boundary after an interrupted stream.
            return self.fail(BluetoothCharacteristicNotificationError::Disconnected, true);
        }
        NotificationDecision::Ignore
    }

    pub(crate) fn terminate(
        &mut self,
        peripheral: BluetoothPeripheralId,
        error: BluetoothCharacteristicNotificationError,
    ) -> NotificationDecision {
        if self.peripheral != Some(peripheral) || !self.is_active() {
            return NotificationDecision::Ignore;
        }
        self.fail(error, false)
    }

    pub(crate) fn callback_panicked(&mut self) -> NotificationDecision {
        if self.is_active() {
            self.callback_ambiguous = true;
            return self.fail(BluetoothCharacteristicNotificationError::Unavailable, true);
        }
        NotificationDecision::Ignore
    }

    pub(crate) fn teardown(&mut self) {
        if self.is_active() {
            self.finish(Phase::Failed);
        }
    }

    fn active_for(&self, peripheral: BluetoothPeripheralId, phase: Phase) -> bool {
        self.peripheral == Some(peripheral) && self.phase == phase
    }

    fn failure(&mut self, error: BluetoothCharacteristicNotificationError) -> NotificationDecision {
        self.last_error = Some(error);
        let (Some(peripheral), Some(service), Some(characteristic)) =
            (self.peripheral, self.service, self.characteristic)
        else {
            return NotificationDecision::Ignore;
        };
        NotificationDecision::Event {
            peripheral,
            service,
            characteristic,
            error,
        }
    }

    fn report_and_finish(
        &mut self,
        error: BluetoothCharacteristicNotificationError,
    ) -> NotificationDecision {
        let decision = self.failure(error);
        self.callback_ambiguous = true;
        self.finish(Phase::Disabled);
        decision
    }

    fn fail(
        &mut self,
        error: BluetoothCharacteristicNotificationError,
        quarantine: bool,
    ) -> NotificationDecision {
        let event = self.failure(error);
        if quarantine {
            self.callback_ambiguous = true;
        }
        self.finish(Phase::Failed);
        event
    }

    fn finish(&mut self, phase: Phase) {
        self.phase = phase;
        self.peripheral = None;
        self.service = None;
        self.characteristic = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> BluetoothPeripheralId {
        BluetoothPeripheralId::from_uuid_bytes([0xB5; 16])
    }

    fn service() -> BluetoothServiceUuid {
        BluetoothServiceUuid::from_uuid_bytes([0x18; 16])
    }

    fn characteristic() -> BluetoothCharacteristicUuid {
        BluetoothCharacteristicUuid::from_uuid_bytes([0x2A; 16])
    }

    fn request(
        connection_state: BluetoothConnectionState,
        native_connected: bool,
        manager_supported: bool,
        service_state: BluetoothServiceDiscoveryState,
        service_match: Result<UniqueMatch, ()>,
    ) -> NotificationRequest {
        NotificationRequest {
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

    fn begin(policy: &mut CharacteristicNotificationPolicy) {
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

    fn enable(policy: &mut CharacteristicNotificationPolicy) {
        begin(policy);
        assert_eq!(
            policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true))),
            NotificationDecision::Enable(0)
        );
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), true, false),
            NotificationDecision::Enabled
        );
    }

    fn failure(error: BluetoothCharacteristicNotificationError) -> NotificationDecision {
        NotificationDecision::Event {
            peripheral: id(),
            service: service(),
            characteristic: characteristic(),
            error,
        }
    }

    #[test]
    fn begin_requires_connected_peer_completed_scoped_discovery_and_unique_service() {
        let cases = [
            (
                request(
                    BluetoothConnectionState::Idle,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::One(0)),
                ),
                BluetoothCharacteristicNotificationError::NotConnected,
            ),
            (
                request(
                    BluetoothConnectionState::Connected,
                    true,
                    false,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::One(0)),
                ),
                BluetoothCharacteristicNotificationError::Unsupported,
            ),
            (
                request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Idle,
                    Ok(UniqueMatch::One(0)),
                ),
                BluetoothCharacteristicNotificationError::ServiceDiscoveryRequired,
            ),
            (
                request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::Missing),
                ),
                BluetoothCharacteristicNotificationError::ServiceNotFound,
            ),
            (
                request(
                    BluetoothConnectionState::Connected,
                    true,
                    true,
                    BluetoothServiceDiscoveryState::Completed,
                    Ok(UniqueMatch::Multiple),
                ),
                BluetoothCharacteristicNotificationError::AmbiguousService,
            ),
        ];
        for (request, error) in cases {
            let mut policy = CharacteristicNotificationPolicy::new();
            assert_eq!(policy.begin(request), Err(error));
        }
        let mut policy = CharacteristicNotificationPolicy::new();
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
            Err(BluetoothCharacteristicNotificationError::ServiceDiscoveryRequired)
        );
    }

    #[test]
    fn characteristic_selection_requires_one_notify_or_indicate_property() {
        for (selection, error) in [
            (
                Ok((UniqueMatch::Missing, false)),
                BluetoothCharacteristicNotificationError::CharacteristicNotFound,
            ),
            (
                Ok((UniqueMatch::Multiple, false)),
                BluetoothCharacteristicNotificationError::AmbiguousCharacteristic,
            ),
            (
                Ok((UniqueMatch::One(0), false)),
                BluetoothCharacteristicNotificationError::NotNotifiable,
            ),
            (
                Err(()),
                BluetoothCharacteristicNotificationError::Unavailable,
            ),
        ] {
            let mut policy = CharacteristicNotificationPolicy::new();
            begin(&mut policy);
            assert_eq!(
                policy.characteristics_discovered(id(), false, selection),
                failure(error)
            );
            assert_eq!(
                policy.state(),
                BluetoothCharacteristicNotificationState::Failed
            );
        }
        let mut policy = CharacteristicNotificationPolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.characteristics_discovered(id(), true, Ok((UniqueMatch::One(0), true))),
            failure(BluetoothCharacteristicNotificationError::NativeFailure)
        );
    }

    #[test]
    fn enable_values_and_disable_follow_confirmed_native_state() {
        let mut policy = CharacteristicNotificationPolicy::new();
        enable(&mut policy);
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Enabled
        );
        assert_eq!(
            policy.value_updated(id(), characteristic(), None),
            Some(service())
        );
        assert_eq!(
            policy.value_updated(
                id(),
                BluetoothCharacteristicUuid::from_uuid_bytes([0x2B; 16]),
                None,
            ),
            None
        );
        assert_eq!(
            policy.begin_disable(),
            Ok((id(), service(), characteristic()))
        );
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Disabling
        );
        assert_eq!(
            policy.value_updated(id(), characteristic(), None),
            Some(service())
        );
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), false, false),
            NotificationDecision::Disabled
        );
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Disabled
        );
        assert_eq!(
            policy.begin_disable(),
            Err(BluetoothCharacteristicNotificationError::NotSubscribed)
        );
    }

    #[test]
    fn confirmed_disable_quarantines_stale_value_callbacks() {
        let mut policy = CharacteristicNotificationPolicy::new();
        enable(&mut policy);
        policy.begin_disable().unwrap();
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), false, false),
            NotificationDecision::Disabled
        );
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Disabled
        );
        assert!(policy.callback_ambiguous());
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicNotificationError::Unavailable)
        );
    }

    #[test]
    fn native_failures_and_oversize_values_are_reported_without_fabricating_success() {
        let mut policy = CharacteristicNotificationPolicy::new();
        begin(&mut policy);
        policy.characteristics_discovered(id(), false, Ok((UniqueMatch::One(0), true)));
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), false, true),
            failure(BluetoothCharacteristicNotificationError::NativeFailure)
        );
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Failed
        );

        let mut policy = CharacteristicNotificationPolicy::new();
        enable(&mut policy);
        policy.begin_disable().unwrap();
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), false, true),
            failure(BluetoothCharacteristicNotificationError::NativeFailure)
        );
        assert_eq!(
            policy.state(),
            BluetoothCharacteristicNotificationState::Disabled
        );
        assert!(policy.callback_ambiguous());

        let mut policy = CharacteristicNotificationPolicy::new();
        enable(&mut policy);
        assert_eq!(
            policy.value_updated(
                id(),
                characteristic(),
                Some(BluetoothCharacteristicNotificationError::ValueTooLarge),
            ),
            Some(service())
        );
        assert_eq!(
            policy.last_error(),
            Some(BluetoothCharacteristicNotificationError::ValueTooLarge)
        );
    }

    #[test]
    fn interrupted_subscription_and_callback_panic_quarantine_stale_callbacks() {
        let mut policy = CharacteristicNotificationPolicy::new();
        begin(&mut policy);
        assert_eq!(
            policy.disconnected(id()),
            failure(BluetoothCharacteristicNotificationError::Disconnected)
        );
        assert_eq!(
            policy.notification_state_changed(id(), characteristic(), true, false),
            NotificationDecision::Ignore
        );
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicNotificationError::Unavailable)
        );

        let mut policy = CharacteristicNotificationPolicy::new();
        enable(&mut policy);
        assert!(matches!(
            policy.callback_panicked(),
            NotificationDecision::Event {
                error: BluetoothCharacteristicNotificationError::Unavailable,
                ..
            }
        ));
        assert_eq!(
            policy.begin(request(
                BluetoothConnectionState::Connected,
                true,
                true,
                BluetoothServiceDiscoveryState::Completed,
                Ok(UniqueMatch::One(0)),
            )),
            Err(BluetoothCharacteristicNotificationError::Unavailable)
        );
    }
}
