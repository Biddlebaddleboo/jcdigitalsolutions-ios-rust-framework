# iOS Watch Connectivity support query

`ios-watch-connectivity` exposes `IosWatchConnectivityBackend`, which implements the portable
`WatchConnectivityBackend` contract with the public `WCSession.isSupported()` class method. The
method result maps directly to `WatchConnectivitySupport::Supported` or
`WatchConnectivitySupport::Unsupported`.

The adapter invokes only `WCSession.isSupported()`. It does not obtain `WCSession.defaultSession`,
call `activateSession`, read `isPaired`, `isWatchAppInstalled`, or `isReachable`, install a
delegate, send a message, or transfer data. Apple documents that a session must be configured and
activated before connection state may be queried or communication attempted; those operations are
outside this crate.

The API is available from iOS 9.0. The backend performs no prompt, user-data access, or permission
request, and makes no entitlement claim. A `Supported` result is not evidence of a paired watch,
an installed counterpart app, an active session, reachability, or successful communication.

The binding dependency is `objc2-watch-connectivity` 0.3.2 with default features disabled and only
`WCSession` enabled. It supplies the typed `WCSession::isSupported()` binding. The crate does not
enable WatchConnectivity file-transfer, user-info-transfer, or block features.

See [the portable capability contract](../capabilities/watch-connectivity.md) and Apple's
[API reference](https://developer.apple.com/documentation/watchconnectivity/wcsession/issupported%28%29).

## Package check gate

From the repository root, run
`platform/ios/ios-watch-connectivity/scripts/check.sh`. It checks package formatting, the
portable `no_std` crate on the host, strict Clippy and rustdoc, device and simulator target
compilation, and the source guard for the status-only native call. It does not run tests or
communicate with a watch. The Apple target checks require Xcode and the `aarch64-apple-ios` and
`aarch64-apple-ios-sim` Rust targets.
