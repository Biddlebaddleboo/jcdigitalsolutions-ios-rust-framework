# Network connectivity snapshot

`framework-connectivity` provides a portable **no_std** contract for one observed local network-path snapshot. It does not perform an endpoint probe, send a request, or start a monitor by itself. A separate backend is selected statically by the caller.

## Rust API

~~~rust
use framework_connectivity::{
    Connectivity, NetworkPathBackend, NetworkPathStatus,
};

async fn show_path<B: NetworkPathBackend>(
    connectivity: &mut Connectivity<B>,
) -> Result<(), framework_connectivity::NetworkPathError> {
    let snapshot = connectivity.current_path().await?;
    match snapshot.status() {
        NetworkPathStatus::Unknown => { /* no valid path status is available */ }
        NetworkPathStatus::Satisfied => { /* a path can attempt a connection */ }
        NetworkPathStatus::Unsatisfied => { /* no currently usable path was reported */ }
        NetworkPathStatus::Satisfiable => { /* a connection attempt may activate a path */ }
        _ => { /* allow future status variants */ }
    }
    Ok(())
}
~~~

The concrete `NetworkPathBackend` type and its associated future are statically selected. There is no boxed trait object, `Send` bound, required executor, registry, or hidden initialization. Constructing the facade or calling `current_path` does not start backend work; the returned future starts no earlier than its first poll. The first non-null native path callback completes the request, including an invalid native status mapped to `Unknown`. No update deadline or synchronous initial snapshot is promised.

## Status semantics and advisory use

- `Satisfied` means only that the backend currently reports a path that can establish a connection attempt. It does not prove Internet access, DNS success, captive-portal clearance, endpoint reachability, or that a request will succeed.
- `Unsatisfied` means the backend reports no currently usable path. `Satisfiable` is distinct: a connection attempt may activate a path. Backends must not collapse it into `Satisfied`.
- `Unknown` represents an invalid or unavailable native status; it does not mean the network is down.
- A snapshot can become stale immediately. Callers must attempt the operation they need and handle its actual result. Never use this snapshot to gate, suppress, or disable a request.

Apple's `NWPathMonitor` is an observer of network path changes; its `satisfied` value is not an endpoint health check. Apple's Developer Technical Support recommends path observations for status UI or retry hints, but not as a preflight gate because false positives and false negatives can block operations that would work. For a URLSession request that should wait for suitable connectivity, use that request's `waitsForConnectivity` behavior instead. This contract does not change the separate `framework-network` HTTP API.

## Lifecycle, errors, and scope

A backend starts native observation no earlier than the first poll, returns the first non-null native path callback once, and stops its native monitor after that callback. A callback with an invalid status still completes the request as `Unknown`. The backend's associated future owns callback state, detachment/invalidation, and native cancellation; dropping the facade future drops that backend future. A generic `framework_async::OperationFuture` only unregisters its waker and does not by itself own native callback detachment or cancellation. The backend future must detach or invalidate callback state before requesting native cancellation when supported, and keep callback state valid until native callbacks can no longer run. Late or duplicate callbacks must not access the dropped future, wake a detached task, or publish another result. No timeout is implied; a caller that needs a deadline must impose one and drop its future.

`NetworkPathError::Backend` preserves the framework `ErrorKind` and optional signed native code. Errors report backend-operation failure; they are not encoded as `Unsatisfied` status.

The snapshot contains only a framework-owned status. It exposes no Apple path/interface/queue/connection object, and the portable contract adds no interface enumeration, link-cost flags, event stream, endpoint check, DNS operation, socket/listener API, or request policy. `PLAN_IOS_CONNECTIVITY.md` owns the separate iOS implementation; this portable crate does not prove live `NWPathMonitor` behavior or device connectivity.

Apple references: [NWPathMonitor](https://developer.apple.com/documentation/network/nwpathmonitor), [NWPath status](https://developer.apple.com/documentation/network/nwpath/status-swift.enum), [Apple Developer Technical Support guidance](https://developer.apple.com/forums/thread/784268), and [URLSession `waitsForConnectivity`](https://developer.apple.com/documentation/foundation/urlsessionconfiguration/waitsforconnectivity).
