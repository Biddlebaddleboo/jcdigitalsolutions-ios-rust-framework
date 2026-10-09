# Watch Connectivity capability

`framework-watch-connectivity` provides a `no_std` portable status contract for whether the
selected platform can provide a Watch Connectivity session object. The iOS implementation is
`ios-watch-connectivity`.

The current contract is intentionally status-only. `WatchConnectivitySupport::Supported` means
the platform can provide a session object. It does not mean a watch is paired, a counterpart app
is installed, a session is active, a device is reachable, or communication will succeed. The
backend does not create or activate a session, prompt the user, communicate with a watch, or make
entitlement claims. It does not provide the broader Watch Connectivity feature set.

For an iOS consumer, import `WatchConnectivityBackend` and
`IosWatchConnectivityBackend`, then call
`IosWatchConnectivityBackend::session_support()`. The backend uses static dispatch and has no
runtime service lookup:

```rust
use framework_watch_connectivity::{WatchConnectivityBackend, WatchConnectivitySupport};
use ios_watch_connectivity::IosWatchConnectivityBackend;

let support = IosWatchConnectivityBackend::session_support();
if support == WatchConnectivitySupport::Supported {
    // The platform can provide a session object; this is not a pairing or reachability result.
}
```

## Scope and limits

- This is a platform/session-object capability query only; pairing, installed-app state,
  activation, reachability, messaging, and transfers are not implemented.
- No pairing, provisioning, entitlement, privacy, or communication behavior is asserted.
- This crate does not define usage-description key or entitlement requirements and does not edit
  host app metadata.
- Apple documents this API for iOS 9.0 and later.

## References

- [Apple `WCSession.isSupported()`](https://developer.apple.com/documentation/watchconnectivity/wcsession/issupported%28%29)
- [Apple `WCSession`](https://developer.apple.com/documentation/watchconnectivity/wcsession)
