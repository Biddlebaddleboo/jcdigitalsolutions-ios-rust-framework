#[path = "../src/dispatch_gate.rs"]
mod dispatch_gate;

use dispatch_gate::DispatchTaskGate;
use std::sync::{Arc, Barrier};

#[test]
fn cancellation_before_start_suppresses_the_callback_claim() {
    let gate = DispatchTaskGate::new();

    assert!(gate.cancel());
    assert!(!gate.try_start());
    assert!(!gate.cancel());
}

#[test]
fn callback_start_wins_against_later_cancellation() {
    let gate = DispatchTaskGate::new();

    assert!(gate.try_start());
    assert!(!gate.cancel());
    gate.finish();
    assert!(!gate.cancel());
}

#[test]
fn cancellation_and_start_have_one_atomic_winner() {
    let gate = Arc::new(DispatchTaskGate::new());
    let barrier = Arc::new(Barrier::new(3));
    let cancel_gate = Arc::clone(&gate);
    let cancel_barrier = Arc::clone(&barrier);
    let cancel = std::thread::spawn(move || {
        cancel_barrier.wait();
        cancel_gate.cancel()
    });
    let start_gate = Arc::clone(&gate);
    let start_barrier = Arc::clone(&barrier);
    let start = std::thread::spawn(move || {
        start_barrier.wait();
        start_gate.try_start()
    });

    barrier.wait();
    let cancelled = cancel.join().unwrap();
    let started = start.join().unwrap();

    assert_ne!(cancelled, started);
    if started {
        gate.finish();
    }
}
