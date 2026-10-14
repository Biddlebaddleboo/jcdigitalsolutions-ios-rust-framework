use std::collections::VecDeque;

pub(crate) const CALLBACK_QUEUE_CAPACITY: usize = 32;

pub(crate) struct CallbackQueue<T> {
    queue: VecDeque<T>,
    dropped: u64,
}

impl<T> CallbackQueue<T> {
    pub(crate) fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(CALLBACK_QUEUE_CAPACITY),
            dropped: 0,
        }
    }

    pub(crate) fn push(&mut self, event: T) -> Option<T> {
        let dropped = if self.queue.len() == CALLBACK_QUEUE_CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            self.queue.pop_front()
        } else {
            None
        };
        self.queue.push_back(event);
        dropped
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    #[cfg(test)]
    pub(crate) fn dropped(&self) -> u64 {
        self.dropped
    }
}

#[cfg(test)]
mod tests {
    use super::{CALLBACK_QUEUE_CAPACITY, CallbackQueue};

    #[test]
    fn callbacks_keep_fifo_order_until_capacity() {
        let mut queue = CallbackQueue::new();
        for event in 0..CALLBACK_QUEUE_CAPACITY {
            assert!(queue.push(event).is_none());
        }
        for event in 0..CALLBACK_QUEUE_CAPACITY {
            assert_eq!(queue.pop(), Some(event));
        }
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.dropped(), 0);
    }

    #[test]
    fn overflow_drops_oldest_and_preserves_newest_fifo_events() {
        let mut queue = CallbackQueue::new();
        for event in 0..=CALLBACK_QUEUE_CAPACITY {
            let dropped = queue.push(event);
            if event == CALLBACK_QUEUE_CAPACITY {
                assert_eq!(dropped, Some(0));
            } else {
                assert_eq!(dropped, None);
            }
        }
        assert_eq!(queue.dropped(), 1);
        for event in 1..=CALLBACK_QUEUE_CAPACITY {
            assert_eq!(queue.pop(), Some(event));
        }
    }
}
