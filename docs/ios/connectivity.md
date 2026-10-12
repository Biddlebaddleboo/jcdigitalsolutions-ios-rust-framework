# iOS network path snapshots and updates

`ios-connectivity` implements the one-shot `framework_connectivity::NetworkPathBackend` and the opt-in `NetworkPathMonitorBackend` with public Network.framework C calls. Each request or caller-owned monitor has its own `nw_path_monitor_t`; no global monitor, executor, service table, URL request, or Swift runtime exists.

```rust,ignore
use framework_connectivity::{Connectivity, NetworkPathStatus};
use ios_connectivity::IosConnectivityBackend;

async fn show_path() -> Result<(), framework_connectivity::NetworkPathError> {
    let mut path = Connectivity::new(IosConnectivityBackend::new());
    let snapshot = path.current_path().await?;
    match snapshot.status() {
        NetworkPathStatus::Unknown => { /* native status is invalid or unavailable */ }
        NetworkPathStatus::Satisfied => { /* a connection attempt has a reported route */ }
        NetworkPathStatus::Unsatisfied => { /* no usable route was reported */ }
        NetworkPathStatus::Satisfiable => { /* a connection attempt may attach a path */ }
        _ => { /* allow a future status variant */ }
    }
    Ok(())
}
```

The caller supplies an executor or polls the future directly. `current_path` does not start native work until its future's first poll. The monitor uses a private serial Dispatch queue; the first non-null update maps the public `nw_path_status_t` value to a portable status and cancels the monitor. `nw_path_status_invalid` and any unrecognized native value map to `Unknown`; that update still completes the future. The C shim's private callback tags are `FRAMEWORK_PATH_STATUS_UNKNOWN = 0`, `FRAMEWORK_PATH_STATUS_SATISFIED = 1`, `FRAMEWORK_PATH_STATUS_UNSATISFIED = 2`, and `FRAMEWORK_PATH_STATUS_SATISFIABLE = 3`; they are not Apple enum values.

For ongoing status hints, use the portable `NetworkPathMonitor` with
`IosNetworkPathMonitorBackend`:

```rust,ignore
use framework_connectivity::NetworkPathMonitor;
use ios_connectivity::IosNetworkPathMonitorBackend;

async fn observe() -> Result<(), framework_connectivity::NetworkPathError> {
    let monitor = NetworkPathMonitor::new(IosNetworkPathMonitorBackend::new());
    monitor.start()?;
    while let Ok(snapshot) = monitor.next_update().await {
        // Treat this status as a hint; attempt operations and handle their actual result
        let _status = snapshot.status();
    }
    Ok(())
}
```

This mode starts only after explicit `start`. It retains one latest pending status and one waiter;
updates that arrive before a consumer poll coalesce to the latest snapshot, so intermediate path
events are not guaranteed. `cancel` wakes a pending waiter with `Cancelled`, and dropping the
monitor cancels it. Both modes expose status only; this increment does not enumerate interfaces
or expose expensive/constrained flags.

`nw_path_monitor_set_cancel_handler` provides the release boundary. The Rust future or backend owns one client reference to a small C shim handle, while the shim retains a second reference until cancellation completes. Native blocks use that handle as callback context. A pending one-shot future drop first marks its completion cell detached, then requests cancel and releases its client reference. The continuous backend marks its event cell cancelled and clears its pending snapshot before requesting cancel; the cancel block releases the callback's `Arc` once after updates can no longer run. A one-shot update requests cancel after its first callback; a continuous update keeps observing until explicit cancellation or drop. The one-shot completion-cell mutex and continuous event-cell mutex prevent late updates from publishing into cancelled state. No callback or Waker retains or accesses a Rust future.

The shim compiles as C with blocks against the installed `<Network/Network.h>` declarations. It converts the SDK enum constants in C, so Rust does not reproduce Apple's numeric enum layout. The `cc` build dependency is local to `ios-connectivity` and exists only to compile this header-checked shim. The backend adds no `.swift` source, Objective-C class, `SCNetworkReachability`, permission request, endpoint check, DNS call, socket, or request gate.

The path is advisory and can change at once. `Satisfied` means only that Network.framework reports a usable route for connection attempts; it does not prove Internet, DNS, captive-portal clearance, endpoint reachability, or success of a particular `URLSession` request. Callers must attempt their operation and handle its real result. This backend does not change the separate `ios-network` HTTP path.

The public Network.framework monitor, status, update, queue, start, cancel, and cancel-handler declarations in the inspected iOS 26.5 SDK have an iOS 12.0 API floor. This is an API floor, not a framework-selected app deployment target. The link-only probe uses iOS 12.0 for device and iOS 14.0 for simulator; `vtool` confirms those minimums and SDK 26.5. `otool -L` reports the direct imports `Network` and `libSystem.B.dylib` on both targets. `nm -u` confirms path-monitor, Dispatch, block, and Rust/libSystem symbol use without Swift, Python, Objective-C class, or unrelated Network capability imports. The compile/link/import gate does not establish live path delivery, cancellation timing, request success, runtime parity, or device connectivity. No physical-device or simulator path-change check is part of this slice.

## Validation status

`sh platform/ios/ios-connectivity/check-link-imports.sh` builds link-only release probes for `aarch64-apple-ios` and `aarch64-apple-ios-sim`; the probe references both one-shot and continuous monitor APIs, checks direct imports and undefined symbols, and does not execute either probe. The exact API, SDK, host, and check results are recorded in `PLAN_IOS_CONNECTIVITY.md` after validation. Neither host tests nor compile/link checks prove live path-change delivery, coalescing under system callbacks, cancellation timing, or device behavior.
