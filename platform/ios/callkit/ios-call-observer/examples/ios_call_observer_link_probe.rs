#![deny(warnings)]

#[cfg(target_os = "ios")]
use ios_call_observer::active_call_snapshot;

#[cfg(target_os = "ios")]
fn main() {
    let snapshot = active_call_snapshot();
    core::hint::black_box((
        snapshot.active_call_count(),
        snapshot.state_flags(),
        snapshot.has_outgoing_call(),
        snapshot.has_connected_call(),
        snapshot.has_call_on_hold(),
        snapshot.has_ended_call(),
    ));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
