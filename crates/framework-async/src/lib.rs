#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Runtime-neutral operation state, cancellation, and a borrowing `Future` adapter."]

mod operation;
mod waker_slot;

pub use operation::{
    CancellationRegistration, CancellationSource, CancellationToken, Completion,
    FutureAlreadyTaken, OperationFuture, OperationPhase, OperationState,
};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::Pin;
    use framework_core::Cancellation;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::task::{Context, Poll, Wake, Waker};

    struct CountWake(AtomicUsize);

    impl Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn context(waker: &Waker) -> Context<'_> {
        Context::from_waker(waker)
    }

    #[test]
    fn future_observes_completion_once_and_wakes_its_task() {
        let state = OperationState::<u32, u8>::new();
        assert!(state.start());
        let mut future = state.future().unwrap();
        let wake = Arc::new(CountWake(AtomicUsize::new(0)));
        let waker = Waker::from(wake.clone());
        let mut cx = context(&waker);
        assert!(matches!(Pin::new(&mut future).poll(&mut cx), Poll::Pending));
        assert!(state.complete(Ok(41)));
        assert!(!state.complete(Ok(42)));
        assert_eq!(wake.0.load(Ordering::SeqCst), 1);
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Completion::Success(41))
        ));
        assert_eq!(state.phase(), OperationPhase::Completed);
        drop(future);
        assert_eq!(state.future().err(), Some(FutureAlreadyTaken));
    }

    #[test]
    fn cancellation_before_start_is_terminal_and_carries_its_reason() {
        let state = OperationState::<(), ()>::new();
        let source = state.cancellation_source();
        let token = source.token();
        assert!(source.cancel(Cancellation::Caller));
        assert!(!state.start());
        assert!(token.is_cancelled());
        assert_eq!(token.reason(), Some(Cancellation::Caller));
        let mut future = state.future().unwrap();
        let mut cx = context(Waker::noop());
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Completion::Cancelled(Cancellation::Caller))
        ));
        drop(future);
        assert_eq!(state.future().err(), Some(FutureAlreadyTaken));
        assert!(!source.cancel(Cancellation::Parent));
    }

    #[test]
    fn dropping_the_future_does_not_cancel_or_destroy_the_operation() {
        let state = OperationState::<u32, ()>::new();
        assert!(state.start());
        drop(state.future().unwrap());
        assert_eq!(state.phase(), OperationPhase::Running);
        let mut future = state.future().unwrap();
        assert!(state.complete(Ok(9)));
        let mut cx = context(Waker::noop());
        assert!(matches!(
            Pin::new(&mut future).poll(&mut cx),
            Poll::Ready(Completion::Success(9))
        ));
    }

    #[test]
    fn only_one_future_can_consume_an_operation_at_a_time() {
        let state = OperationState::<(), ()>::new();
        let future = state.future().unwrap();
        assert_eq!(state.future().err(), Some(FutureAlreadyTaken));
        drop(future);
        assert!(state.future().is_ok());
    }

    #[test]
    fn controlled_cancel_completion_race_has_exactly_one_winner() {
        let state = Arc::new(OperationState::<u32, ()>::new());
        assert!(state.start());
        let barrier = Arc::new(Barrier::new(3));
        let completion_state = state.clone();
        let completion_barrier = barrier.clone();
        let completion = std::thread::spawn(move || {
            completion_barrier.wait();
            completion_state.complete(Ok(7))
        });
        let cancel_state = state.clone();
        let cancel_barrier = barrier.clone();
        let cancellation = std::thread::spawn(move || {
            cancel_barrier.wait();
            cancel_state
                .cancellation_source()
                .cancel(Cancellation::Parent)
        });
        barrier.wait();
        let completed = completion.join().unwrap();
        let cancelled = cancellation.join().unwrap();
        assert_ne!(completed, cancelled);
        let mut future = state.future().unwrap();
        let mut cx = context(Waker::noop());
        match (completed, cancelled, Pin::new(&mut future).poll(&mut cx)) {
            (true, false, Poll::Ready(Completion::Success(7))) => {
                assert_eq!(state.phase(), OperationPhase::Completed)
            }
            (false, true, Poll::Ready(Completion::Cancelled(Cancellation::Parent))) => {
                assert_eq!(state.phase(), OperationPhase::Cancelled)
            }
            result => panic!("unexpected operation race result: {result:?}"),
        }
    }
}
