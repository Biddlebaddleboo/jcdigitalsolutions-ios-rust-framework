use framework_bluetooth::BluetoothDiscovery;

/// Fixed-capacity discovery queue; overflow drops the newest event and saturates the counter.
pub(crate) struct DiscoveryQueue {
    events: [Option<BluetoothDiscovery>; Self::CAPACITY],
    head: usize,
    len: usize,
    pub(crate) dropped: u32,
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

#[cfg(test)]
mod tests {
    use super::*;
    use framework_bluetooth::BluetoothPeripheralId;

    fn event(value: u8) -> BluetoothDiscovery {
        BluetoothDiscovery {
            peripheral: BluetoothPeripheralId::from_uuid_bytes([value; 16]),
            rssi_dbm: Some(i32::from(value)),
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
}
