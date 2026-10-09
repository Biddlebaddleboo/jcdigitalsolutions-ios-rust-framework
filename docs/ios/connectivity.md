# iOS network path snapshot

`ios-connectivity` implements `framework_connectivity::NetworkPathBackend` with public Network.framework C calls. Each request owns one `nw_path_monitor_t`; no global monitor, executor, service table, URL request, or Swift runtime exists.

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

`nw_path_monitor_set_cancel_handler` provides the release boundary. The Rust future owns one client reference to a small C shim handle, while the shim retains a second reference until cancellation completes. The native update and cancel blocks use that handle as callback context. A pending future drop first marks the Rust waker/result cell detached, then requests cancel and releases its client reference. The cancel block runs on the configured serial queue after cancellation and after the update block can no longer run; it releases the callback's `Arc` once, then releases the shim's operation reference. A first update also requests cancel. An atomic once flag serializes that request against future drop. The completion cell mutex sets the race order: if an update claims completion before detach, that accepted completion may call its already-claimed Waker after drop; if detach claims the incomplete cell first, later updates cannot publish a result or wake the task. No callback or Waker retains or accesses the Rust future.

The shim compiles as C with blocks against the installed `<Network/Network.h>` declarations. It converts the SDK enum constants in C, so Rust does not reproduce Apple's numeric enum layout. The `cc` build dependency is local to `ios-connectivity` and exists only to compile this header-checked shim. The backend adds no `.swift` source, Objective-C class, `SCNetworkReachability`, permission request, endpoint check, DNS call, socket, or request gate.

The path is advisory and can change at once. `Satisfied` means only that Network.framework reports a usable route for connection attempts; it does not prove Internet, DNS, captive-portal clearance, endpoint reachability, or success of a particular `URLSession` request. Callers must attempt their operation and handle its real result. This backend does not change the separate `ios-network` HTTP path.

The public Network.framework monitor, status, update, queue, start, cancel, and cancel-handler declarations in the inspected iOS 26.5 SDK have an iOS 12.0 API floor. This is an API floor, not a framework-selected app deployment target. The link-only probe uses iOS 12.0 for device and iOS 14.0 for simulator; `vtool` confirms those minimums and SDK 26.5. `otool -L` reports the direct imports `Network` and `libSystem.B.dylib` on both targets. `nm -u` confirms path-monitor, Dispatch, block, and Rust/libSystem symbol use without Swift, Python, Objective-C class, or unrelated Network capability imports. The compile/link/import gate does not establish live path delivery, cancellation timing, request success, runtime parity, or device connectivity. No physical-device or simulator path-change check is part of this slice.

## Validation status

`sh platform/ios/ios-connectivity/check-link-imports.sh` builds link-only release probes for `aarch64-apple-ios` and `aarch64-apple-ios-sim`; it checks direct imports and undefined symbols and does not execute either probe. The exact API, SDK, host, and check results are recorded in `PLAN_IOS_CONNECTIVITY.md` after validation.
