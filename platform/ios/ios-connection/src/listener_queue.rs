use std::collections::VecDeque;

use framework_connection::MAX_QUEUED_ACCEPTS;

pub(crate) enum QueuePushError<T> {
    Full(T),
    ResourceExhausted(T),
}

pub(crate) struct PendingAcceptQueue<T> {
    entries: VecDeque<T>,
}

impl<T> PendingAcceptQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    pub(crate) fn push(&mut self, entry: T) -> Result<(), QueuePushError<T>> {
        if self.entries.len() >= MAX_QUEUED_ACCEPTS {
            return Err(QueuePushError::Full(entry));
        }
        if self.entries.try_reserve(1).is_err() {
            return Err(QueuePushError::ResourceExhausted(entry));
        }
        self.entries.push_back(entry);
        Ok(())
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        self.entries.pop_front()
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn take_all(&mut self) -> VecDeque<T> {
        core::mem::take(&mut self.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_queue_preserves_fifo_and_rejects_overflow_item() {
        let mut queue = PendingAcceptQueue::new();
        for value in 0..MAX_QUEUED_ACCEPTS {
            assert!(queue.push(value).is_ok());
        }
        assert!(matches!(
            queue.push(MAX_QUEUED_ACCEPTS),
            Err(QueuePushError::Full(MAX_QUEUED_ACCEPTS))
        ));
        assert_eq!(queue.len(), MAX_QUEUED_ACCEPTS);
        for value in 0..MAX_QUEUED_ACCEPTS {
            assert_eq!(queue.pop(), Some(value));
        }
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn queue_clear_returns_owned_entries_for_drop_outside_lock() {
        let mut queue = PendingAcceptQueue::new();
        assert!(queue.push(String::from("pending")).is_ok());
        let mut pending = queue.take_all();
        assert_eq!(queue.len(), 0);
        assert_eq!(pending.pop_front().as_deref(), Some("pending"));
    }
}
