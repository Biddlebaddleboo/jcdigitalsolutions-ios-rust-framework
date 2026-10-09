# Personal VPN status

`framework-vpn` provides portable owned values for a narrow iOS query: the status of the calling
app's Personal VPN connection after NetworkExtension loads that app's preferences. `ios-vpn`
adapts this contract with `NEVPNManager.sharedManager()`,
`loadFromPreferencesWithCompletionHandler:`, and `NEVPNConnection.status`.

This is a partial capability only. The host app needs the Personal VPN entitlement
`com.apple.developer.networking.vpn.api` with the `allow-vpn` value and Apple's approval where
required. A Rust compile or link does not establish that an app has the entitlement or may use it.
The queried Apple API floor is iOS 8.0. With rustc 1.94.1, the Rust toolchain's minimum supported
iOS deployment target is 10.0; this package does not claim a linked Rust app can target iOS 8 or 9.

The result describes only the calling app's Personal VPN profile at one query time. `Invalid` is
preserved as `NEVPNStatusInvalid`; it does not mean that no VPN is active device-wide. A preference
load error keeps its native error domain and integer code and is not mapped to a disconnected
status. Unknown future status values keep their raw integer.

The adapter only loads preferences and reads the connection status. It does not inspect or mutate
configuration properties, save or remove a profile, start or stop a tunnel, implement a provider,
or report routes, reachability, tunnel health, or system-wide VPN state. The preference-load
completion runs on the caller's main thread. Callers must construct, poll, and drop the returned
future on that thread using `ios_runtime::main_thread::MainThread`; the native request begins when
the request function runs. Dropping the future detaches Rust interest but does not cancel the
native request.

See the [iOS status guide](../ios/vpn-status.md), [`framework-vpn`](../../crates/framework-vpn/README.md),
[`ios-vpn`](../../platform/ios/ios-vpn/README.md), and [B79 plan](../../PLAN_IOS_VPN_STATUS.md).
