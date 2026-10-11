use core::task::{Context, Poll, Waker};
use framework_location::{LocationUpdate, LocationUpdateKind};
use std::sync::{Mutex, MutexGuard};

/// One native callback result and the future's current interest in it.
pub(crate) struct CompletionCell<T, E> {
    state: Mutex<State<T, E>>,
}

struct State<T, E> {
    completed: bool,
    detached: bool,
    result: Option<Result<T, E>>,
    waker: Option<Waker>,
}

impl<T, E> CompletionCell<T, E> {
    /// Creates an incomplete cell with no callback or future interest.
    pub(crate) const fn new() -> Self {
        Self {
            state: Mutex::new(State {
                completed: false,
                detached: false,
                result: None,
                waker: None,
            }),
        }
    }

    /// Publishes one terminal callback result and wakes an interested future.
    pub(crate) fn complete(&self, result: Result<T, E>) -> bool {
        let waker = {
            let mut state = self.lock();
            if state.completed {
                return false;
            }
            state.completed = true;
            if state.detached {
                None
            } else {
                state.result = Some(result);
                state.waker.take()
            }
        };
        if let Some(waker) = waker {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
        }
        true
    }

    /// Polls for one result or saves the current task's waker.
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<Result<T, E>> {
        let mut state = self.lock();
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
        }
        if !state.completed
            && !state.detached
            && state
                .waker
                .as_ref()
                .is_none_or(|waker| !waker.will_wake(context.waker()))
        {
            state.waker = Some(context.waker().clone());
        }
        Poll::Pending
    }

    /// Drops future interest but leaves callback state valid until its owner is released.
    pub(crate) fn detach(&self) {
        let mut state = self.lock();
        state.detached = true;
        state.result = None;
        state.waker = None;
    }

    fn lock(&self) -> MutexGuard<'_, State<T, E>> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Capacity for the bounded foreground location event queue.
pub(crate) const UPDATE_QUEUE_CAPACITY: usize = 16;

/// A bounded event queue shared by the location delegate and its session.
pub(crate) struct LocationUpdateQueue {
    state: Mutex<UpdateState>,
}

struct UpdateState {
    events: [Option<LocationUpdate>; UPDATE_QUEUE_CAPACITY],
    head: usize,
    len: usize,
    missed_events: u64,
    closed: bool,
    waker: Option<Waker>,
}

impl LocationUpdateQueue {
    /// Creates an empty event queue for one update session.
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(UpdateState {
                events: [None; UPDATE_QUEUE_CAPACITY],
                head: 0,
                len: 0,
                missed_events: 0,
                closed: false,
                waker: None,
            }),
        }
    }

    /// Adds a batch in iterator order, making any bounded-queue eviction observable.
    pub(crate) fn push_batch<I>(&self, events: I)
    where
        I: IntoIterator<Item = LocationUpdateKind>,
    {
        for event in events {
            self.push(event);
        }
    }

    /// Polls for the oldest retained event and records the current task's waker if empty.
    pub(crate) fn poll(&self, context: &mut Context<'_>) -> Poll<LocationUpdate> {
        let mut new_waker = None;
        loop {
            let mut state = self.lock();
            if state.len != 0 {
                let head = state.head;
                let event = state.events[head]
                    .take()
                    .expect("occupied queue slot contains an event");
                state.head = (head + 1) % UPDATE_QUEUE_CAPACITY;
                state.len -= 1;
                let missed_events = core::mem::replace(&mut state.missed_events, 0);
                drop(state);
                drop(new_waker);
                return Poll::Ready(LocationUpdate::new(event.kind(), missed_events));
            }
            if state.closed {
                drop(state);
                drop(new_waker);
                return Poll::Pending;
            }
            if state
                .waker
                .as_ref()
                .is_some_and(|waker| waker.will_wake(context.waker()))
            {
                drop(state);
                drop(new_waker);
                return Poll::Pending;
            }
            if let Some(new_waker) = new_waker.take() {
                let previous_waker = state.waker.replace(new_waker);
                drop(state);
                drop(previous_waker);
                return Poll::Pending;
            }
            drop(state);
            new_waker = Some(context.waker().clone());
        }
    }

    /// Clears only the matching waiter's waker when its next-event future is dropped.
    pub(crate) fn clear_waker(&self, waker: &Waker) {
        let previous_waker = {
            let mut state = self.lock();
            if state
                .waker
                .as_ref()
                .is_some_and(|registered| registered.will_wake(waker))
            {
                state.waker.take()
            } else {
                None
            }
        };
        drop(previous_waker);
    }

    /// Closes the queue and detaches any pending waiter during session drop.
    pub(crate) fn detach(&self) {
        let previous_waker = {
            let mut state = self.lock();
            state.closed = true;
            state.events.fill(None);
            state.head = 0;
            state.len = 0;
            state.waker.take()
        };
        drop(previous_waker);
    }

    fn push(&self, kind: LocationUpdateKind) {
        let waker = {
            let mut state = self.lock();
            if state.closed {
                return;
            }
            if state.len == UPDATE_QUEUE_CAPACITY {
                let head = state.head;
                state.events[head] = None;
                state.head = (head + 1) % UPDATE_QUEUE_CAPACITY;
                state.len -= 1;
                state.missed_events = state.missed_events.saturating_add(1);
            }
            let index = (state.head + state.len) % UPDATE_QUEUE_CAPACITY;
            state.events[index] = Some(LocationUpdate::new(kind, 0));
            state.len += 1;
            state.waker.take()
        };
        if let Some(waker) = waker {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()));
        }
    }

    fn lock(&self) -> MutexGuard<'_, UpdateState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use framework_core::{Error, ErrorKind};
    use framework_location::{Coordinate, LocationError, LocationFix, LocationTimestamp};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{RawWaker, RawWakerVTable, Wake, Waker};

    struct WakeCount(AtomicUsize);

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct LockCheck<'a> {
        queue: &'a LocationUpdateQueue,
        dropped_while_locked: &'a AtomicUsize,
    }

    unsafe fn clone_lock_check_waker(data: *const ()) -> RawWaker {
        // SAFETY: The test keeps its stack-allocated probe alive until all waker clones have been
        // dropped, and cloning is checked outside the queue lock.
        let probe = unsafe { &*data.cast::<LockCheck<'_>>() };
        if probe.queue.state.try_lock().is_err() {
            probe.dropped_while_locked.fetch_add(1, Ordering::SeqCst);
        }
        RawWaker::new(data, &LOCK_CHECK_WAKER_VTABLE)
    }

    unsafe fn wake_lock_check_waker(data: *const ()) {
        // SAFETY: This test waker stores a non-owning pointer that remains live until all clones
        // have been dropped; waking consumes one clone and uses the same drop probe.
        unsafe { drop_lock_check_waker(data) };
    }

    unsafe fn wake_lock_check_waker_by_ref(_data: *const ()) {}

    unsafe fn drop_lock_check_waker(data: *const ()) {
        // SAFETY: The test keeps its stack-allocated probe alive until the queue and local wakers
        // have been cleared or dropped.
        let probe = unsafe { &*data.cast::<LockCheck<'_>>() };
        if probe.queue.state.try_lock().is_err() {
            probe.dropped_while_locked.fetch_add(1, Ordering::SeqCst);
        }
    }

    static LOCK_CHECK_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone_lock_check_waker,
        wake_lock_check_waker,
        wake_lock_check_waker_by_ref,
        drop_lock_check_waker,
    );

    fn lock_check_waker(probe: &LockCheck<'_>) -> Waker {
        let raw = RawWaker::new(
            probe as *const LockCheck<'_> as *const (),
            &LOCK_CHECK_WAKER_VTABLE,
        );
        // SAFETY: The custom test vtable treats its data pointer as non-owning, and this test
        // clears queue-held clones before the probe leaves scope.
        unsafe { Waker::from_raw(raw) }
    }

    #[test]
    fn completion_is_exactly_once_and_wakes_after_unlock() {
        let cell = CompletionCell::<u8, u8>::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut context = Context::from_waker(&waker);
        assert!(cell.poll(&mut context).is_pending());
        assert!(cell.complete(Ok(7)));
        assert!(!cell.complete(Ok(8)));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 1);
        assert!(matches!(cell.poll(&mut context), Poll::Ready(Ok(7))));
    }

    #[test]
    fn detached_future_discards_late_callback_without_waking() {
        let cell = CompletionCell::<u8, u8>::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut context = Context::from_waker(&waker);
        assert!(cell.poll(&mut context).is_pending());
        cell.detach();
        assert!(cell.complete(Ok(9)));
        assert!(!cell.complete(Ok(10)));
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
    }

    fn fix(latitude: f64) -> LocationUpdateKind {
        let coordinate = Coordinate::new(latitude, 0.0).unwrap();
        let fix =
            LocationFix::new(coordinate, 1.0, LocationTimestamp::from_unix_millis(1)).unwrap();
        LocationUpdateKind::Fix(fix)
    }

    fn poll(queue: &LocationUpdateQueue) -> Poll<LocationUpdate> {
        queue.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn update_queue_preserves_callback_array_order() {
        let queue = LocationUpdateQueue::new();
        queue.push_batch([fix(1.0), fix(2.0), fix(3.0)]);
        for latitude in [1.0, 2.0, 3.0] {
            let Poll::Ready(event) = poll(&queue) else {
                panic!("queued update was not ready");
            };
            assert_eq!(event.missed_before(), 0);
            assert_eq!(
                event.kind(),
                LocationUpdateKind::Fix(
                    LocationFix::new(
                        Coordinate::new(latitude, 0.0).unwrap(),
                        1.0,
                        LocationTimestamp::from_unix_millis(1),
                    )
                    .unwrap()
                )
            );
        }
    }

    #[test]
    fn update_queue_evicts_oldest_and_reports_exact_missed_count() {
        let queue = LocationUpdateQueue::new();
        queue.push_batch((0..=UPDATE_QUEUE_CAPACITY).map(|value| fix(value as f64)));
        let Poll::Ready(first) = poll(&queue) else {
            panic!("retained update was not ready");
        };
        assert_eq!(first.missed_before(), 1);
        assert_eq!(
            first.kind(),
            fix(1.0),
            "the oldest update must be evicted first"
        );
        for value in 2..=UPDATE_QUEUE_CAPACITY {
            let Poll::Ready(event) = poll(&queue) else {
                panic!("retained update was not ready");
            };
            assert_eq!(event.missed_before(), 0);
            assert_eq!(event.kind(), fix(value as f64));
        }
    }

    #[test]
    fn update_queue_overflow_count_accumulates_before_next_retained_event() {
        let queue = LocationUpdateQueue::new();
        queue.push_batch((0..(UPDATE_QUEUE_CAPACITY * 3)).map(|value| fix(value as f64)));
        let Poll::Ready(first) = poll(&queue) else {
            panic!("retained update was not ready");
        };
        assert_eq!(first.missed_before(), (UPDATE_QUEUE_CAPACITY * 2) as u64);
        assert_eq!(first.kind(), fix((UPDATE_QUEUE_CAPACITY * 2) as f64));
    }

    #[test]
    fn update_queue_missed_event_count_saturates_at_u64_max() {
        let queue = LocationUpdateQueue::new();
        queue.lock().missed_events = u64::MAX;
        queue.push_batch((0..=UPDATE_QUEUE_CAPACITY).map(|value| fix(value as f64)));
        let Poll::Ready(first) = poll(&queue) else {
            panic!("retained update was not ready");
        };
        assert_eq!(first.missed_before(), u64::MAX);
        assert_eq!(first.kind(), fix(1.0));
    }

    #[test]
    fn update_queue_delivers_nonterminal_error_events_in_order() {
        let queue = LocationUpdateQueue::new();
        let error = LocationError::Backend(Error::new(ErrorKind::Unavailable));
        queue.push_batch([
            LocationUpdateKind::Error(error),
            fix(7.0),
            LocationUpdateKind::Error(error),
        ]);
        for expected in [
            LocationUpdateKind::Error(error),
            fix(7.0),
            LocationUpdateKind::Error(error),
        ] {
            let Poll::Ready(event) = poll(&queue) else {
                panic!("queued event was not ready");
            };
            assert_eq!(event.kind(), expected);
        }
    }

    #[test]
    fn update_queue_waiter_cancel_clears_only_waker_and_keeps_session_open() {
        let queue = LocationUpdateQueue::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut context = Context::from_waker(&waker);
        assert!(queue.poll(&mut context).is_pending());
        queue.clear_waker(&waker);
        queue.push_batch([fix(8.0)]);
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
        assert!(matches!(poll(&queue), Poll::Ready(_)));
        assert!(poll(&queue).is_pending());
    }

    #[test]
    fn update_queue_drops_replaced_cleared_and_detached_wakers_after_unlock() {
        let queue = LocationUpdateQueue::new();
        let dropped_while_locked = AtomicUsize::new(0);
        let probe = LockCheck {
            queue: &queue,
            dropped_while_locked: &dropped_while_locked,
        };
        let waker = lock_check_waker(&probe);
        let mut context = Context::from_waker(&waker);
        assert!(queue.poll(&mut context).is_pending());
        assert!(
            queue
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert_eq!(dropped_while_locked.load(Ordering::SeqCst), 0);

        let queue = LocationUpdateQueue::new();
        let probe = LockCheck {
            queue: &queue,
            dropped_while_locked: &dropped_while_locked,
        };
        let waker = lock_check_waker(&probe);
        let mut context = Context::from_waker(&waker);
        assert!(queue.poll(&mut context).is_pending());
        queue.clear_waker(&waker);
        assert_eq!(dropped_while_locked.load(Ordering::SeqCst), 0);

        let queue = LocationUpdateQueue::new();
        let probe = LockCheck {
            queue: &queue,
            dropped_while_locked: &dropped_while_locked,
        };
        let waker = lock_check_waker(&probe);
        let mut context = Context::from_waker(&waker);
        assert!(queue.poll(&mut context).is_pending());
        queue.detach();
        assert_eq!(dropped_while_locked.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn update_queue_detach_ignores_late_callbacks_and_waiters() {
        let queue = LocationUpdateQueue::new();
        let wake_count = Arc::new(WakeCount(AtomicUsize::new(0)));
        let waker = Waker::from(wake_count.clone());
        let mut context = Context::from_waker(&waker);
        assert!(queue.poll(&mut context).is_pending());
        queue.detach();
        queue.push_batch([fix(9.0)]);
        assert_eq!(wake_count.0.load(Ordering::SeqCst), 0);
        assert!(poll(&queue).is_pending());
    }
}
