use framework_bluetooth::{
    BluetoothCharacteristicNotificationEvent, BluetoothCharacteristicReadEvent,
    BluetoothCharacteristicWriteEvent, BluetoothConnectionEvent, BluetoothDiscovery,
    BluetoothServiceDiscoveryEvent,
};

/// Fixed-capacity discovery queue; overflow drops the newest event and saturates the counter.
pub(crate) struct DiscoveryQueue {
    events: [Option<BluetoothDiscovery>; Self::CAPACITY],
    head: usize,
    len: usize,
    pub(crate) dropped: u32,
}

/// Fixed-capacity connection-event FIFO; overflow drops the newest event.
pub(crate) struct ConnectionQueue {
    events: [Option<BluetoothConnectionEvent>; Self::CAPACITY],
    head: usize,
    len: usize,
    dropped: u32,
}

/// Fixed-capacity GATT service-discovery FIFO; overflow drops newest events.
pub(crate) struct ServiceDiscoveryQueue {
    events: [Option<BluetoothServiceDiscoveryEvent>; Self::CAPACITY],
    head: usize,
    len: usize,
    dropped: u32,
}

/// Fixed-capacity read-result FIFO; overflow drops newest events.
pub(crate) struct CharacteristicReadQueue {
    events: [Option<BluetoothCharacteristicReadEvent>; Self::CAPACITY],
    head: usize,
    len: usize,
    dropped: u32,
}

/// Fixed-capacity write-result FIFO; overflow drops newest events.
pub(crate) struct CharacteristicWriteQueue {
    events: [Option<BluetoothCharacteristicWriteEvent>; Self::CAPACITY],
    head: usize,
    len: usize,
    dropped: u32,
}

/// Fixed-capacity notification-value FIFO; overflow drops newest events.
pub(crate) struct CharacteristicNotificationQueue {
    events: [Option<BluetoothCharacteristicNotificationEvent>; Self::CAPACITY],
    head: usize,
    len: usize,
    dropped: u32,
}

impl ConnectionQueue {
    pub(crate) const CAPACITY: usize = 16;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothConnectionEvent) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothConnectionEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

impl DiscoveryQueue {
    pub(crate) const CAPACITY: usize = 32;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothDiscovery) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothDiscovery> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

impl ServiceDiscoveryQueue {
    pub(crate) const CAPACITY: usize = 32;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothServiceDiscoveryEvent) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothServiceDiscoveryEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

impl CharacteristicReadQueue {
    pub(crate) const CAPACITY: usize = 4;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothCharacteristicReadEvent) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothCharacteristicReadEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

impl CharacteristicWriteQueue {
    pub(crate) const CAPACITY: usize = 4;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothCharacteristicWriteEvent) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothCharacteristicWriteEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

impl CharacteristicNotificationQueue {
    pub(crate) const CAPACITY: usize = 4;

    pub(crate) const fn new() -> Self {
        Self {
            events: [None; Self::CAPACITY],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: BluetoothCharacteristicNotificationEvent) {
        if self.len == Self::CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let tail = (self.head + self.len) % Self::CAPACITY;
        self.events[tail] = Some(event);
        self.len += 1;
    }

    pub(crate) fn pop(&mut self) -> Option<BluetoothCharacteristicNotificationEvent> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head].take();
        self.head = (self.head + 1) % Self::CAPACITY;
        self.len -= 1;
        event
    }

    pub(crate) fn take_dropped(&mut self) -> u32 {
        let dropped = self.dropped;
        self.dropped = 0;
        dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_bluetooth::{
        BluetoothCharacteristicNotificationError, BluetoothCharacteristicNotificationEvent,
        BluetoothCharacteristicReadEvent, BluetoothCharacteristicUuid,
        BluetoothCharacteristicValue, BluetoothCharacteristicWriteError,
        BluetoothCharacteristicWriteEvent, BluetoothConnectionState, BluetoothPeripheralId,
        BluetoothServiceUuid,
    };

    fn event(value: u8) -> BluetoothDiscovery {
        BluetoothDiscovery {
            peripheral: BluetoothPeripheralId::from_uuid_bytes([value; 16]),
            rssi_dbm: Some(i32::from(value)),
        }
    }

    fn connection_event(value: u8) -> BluetoothConnectionEvent {
        BluetoothConnectionEvent::StateChanged {
            peripheral: BluetoothPeripheralId::from_uuid_bytes([value; 16]),
            state: BluetoothConnectionState::Connecting,
        }
    }

    #[test]
    fn queue_preserves_fifo_and_drops_newest_when_full() {
        let mut queue = DiscoveryQueue::new();
        for value in 0..DiscoveryQueue::CAPACITY as u8 {
            queue.push(event(value));
        }
        queue.push(event(255));
        assert_eq!(queue.dropped, 1);
        assert_eq!(queue.take_dropped(), 1);
        assert_eq!(queue.dropped, 0);
        for value in 0..DiscoveryQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(event(value)));
        }
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn queue_wraps_after_pop_and_reports_saturating_overflow_count() {
        let mut queue = DiscoveryQueue::new();
        for value in 0..DiscoveryQueue::CAPACITY as u8 {
            queue.push(event(value));
        }
        assert_eq!(queue.pop(), Some(event(0)));
        queue.push(event(42));
        assert_eq!(queue.pop(), Some(event(1)));
        assert_eq!(queue.pop(), Some(event(2)));
        for value in 3..DiscoveryQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(event(value)));
        }
        assert_eq!(queue.pop(), Some(event(42)));
        for value in 0..DiscoveryQueue::CAPACITY as u8 {
            queue.push(event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(event(43));
        assert_eq!(queue.dropped, u32::MAX);
    }

    #[test]
    fn connection_queue_is_bounded_fifo_and_drops_newest() {
        let mut queue = ConnectionQueue::new();
        for value in 0..ConnectionQueue::CAPACITY as u8 {
            queue.push(connection_event(value));
        }
        queue.push(connection_event(255));
        assert_eq!(queue.take_dropped(), 1);
        for value in 0..ConnectionQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(connection_event(value)));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.take_dropped(), 0);
        for value in 0..ConnectionQueue::CAPACITY as u8 {
            queue.push(connection_event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(connection_event(42));
        assert_eq!(queue.dropped, u32::MAX);
    }

    #[test]
    fn service_discovery_queue_is_bounded_fifo_and_drops_newest() {
        let mut queue = ServiceDiscoveryQueue::new();
        for value in 0..ServiceDiscoveryQueue::CAPACITY as u8 {
            queue.push(service_event(value));
        }
        queue.push(BluetoothServiceDiscoveryEvent::Completed { peripheral: id(99) });
        assert_eq!(queue.take_dropped(), 1);
        for value in 0..ServiceDiscoveryQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(service_event(value)));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.take_dropped(), 0);
        for value in 0..ServiceDiscoveryQueue::CAPACITY as u8 {
            queue.push(service_event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(service_event(99));
        assert_eq!(queue.take_dropped(), u32::MAX);
    }

    fn id(value: u8) -> BluetoothPeripheralId {
        BluetoothPeripheralId::from_uuid_bytes([value; 16])
    }

    fn service_event(value: u8) -> BluetoothServiceDiscoveryEvent {
        BluetoothServiceDiscoveryEvent::ServiceDiscovered {
            peripheral: id(1),
            service: BluetoothServiceUuid::from_uuid_bytes([value; 16]),
        }
    }

    #[test]
    fn characteristic_read_queue_is_bounded_fifo_and_drops_newest() {
        let mut queue = CharacteristicReadQueue::new();
        for value in 0..CharacteristicReadQueue::CAPACITY as u8 {
            queue.push(read_event(value));
        }
        queue.push(read_event(255));
        assert_eq!(queue.take_dropped(), 1);
        for value in 0..CharacteristicReadQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(read_event(value)));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.take_dropped(), 0);
        for value in 0..CharacteristicReadQueue::CAPACITY as u8 {
            queue.push(read_event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(read_event(42));
        assert_eq!(queue.take_dropped(), u32::MAX);
    }

    #[test]
    fn characteristic_write_queue_is_bounded_fifo_and_drops_newest() {
        let mut queue = CharacteristicWriteQueue::new();
        for value in 0..CharacteristicWriteQueue::CAPACITY as u8 {
            queue.push(write_event(value));
        }
        queue.push(write_event(255));
        assert_eq!(queue.take_dropped(), 1);
        for value in 0..CharacteristicWriteQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(write_event(value)));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.take_dropped(), 0);
        for value in 0..CharacteristicWriteQueue::CAPACITY as u8 {
            queue.push(write_event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(write_event(42));
        assert_eq!(queue.take_dropped(), u32::MAX);
    }

    #[test]
    fn characteristic_notification_queue_is_bounded_fifo_and_drops_newest() {
        let mut queue = CharacteristicNotificationQueue::new();
        for value in 0..CharacteristicNotificationQueue::CAPACITY as u8 {
            queue.push(notification_event(value));
        }
        queue.push(notification_event(255));
        assert_eq!(queue.take_dropped(), 1);
        for value in 0..CharacteristicNotificationQueue::CAPACITY as u8 {
            assert_eq!(queue.pop(), Some(notification_event(value)));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.take_dropped(), 0);
        for value in 0..CharacteristicNotificationQueue::CAPACITY as u8 {
            queue.push(notification_event(value));
        }
        queue.dropped = u32::MAX;
        queue.push(notification_event(42));
        assert_eq!(queue.take_dropped(), u32::MAX);
    }

    fn read_event(value: u8) -> BluetoothCharacteristicReadEvent {
        BluetoothCharacteristicReadEvent::Completed {
            peripheral: id(1),
            service: BluetoothServiceUuid::from_uuid_bytes([value; 16]),
            characteristic: BluetoothCharacteristicUuid::from_uuid_bytes([value; 16]),
            value: BluetoothCharacteristicValue::from_bytes(&[value]).unwrap(),
        }
    }

    fn write_event(value: u8) -> BluetoothCharacteristicWriteEvent {
        if value % 2 == 0 {
            BluetoothCharacteristicWriteEvent::Completed {
                peripheral: id(1),
                service: BluetoothServiceUuid::from_uuid_bytes([value; 16]),
                characteristic: BluetoothCharacteristicUuid::from_uuid_bytes([value; 16]),
            }
        } else {
            BluetoothCharacteristicWriteEvent::Failed {
                peripheral: id(1),
                service: BluetoothServiceUuid::from_uuid_bytes([value; 16]),
                characteristic: BluetoothCharacteristicUuid::from_uuid_bytes([value; 16]),
                error: BluetoothCharacteristicWriteError::NativeFailure,
            }
        }
    }

    fn notification_event(value: u8) -> BluetoothCharacteristicNotificationEvent {
        BluetoothCharacteristicNotificationEvent::new(
            id(1),
            BluetoothServiceUuid::from_uuid_bytes([value; 16]),
            BluetoothCharacteristicUuid::from_uuid_bytes([value; 16]),
            if value % 2 == 0 {
                Ok(BluetoothCharacteristicValue::from_bytes(&[value]).unwrap())
            } else {
                Err(BluetoothCharacteristicNotificationError::NativeFailure)
            },
        )
    }
}
