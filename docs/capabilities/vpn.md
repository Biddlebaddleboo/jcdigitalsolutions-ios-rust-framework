# Personal VPN status and configuration

`framework-vpn` provides portable owned values for two narrow iOS queries after NetworkExtension
loads the calling app's Personal VPN preferences. `ios-vpn` reads the connection's
`NEVPNConnection.status` or the configuration flags `NEVPNManager.isEnabled` and
`NEVPNManager.isOnDemandEnabled`.

This is a partial capability only. The host app needs the Personal VPN entitlement
`com.apple.developer.networking.vpn.api` with the `allow-vpn` value and Apple's approval where
required. A Rust compile or link does not establish that an app has the entitlement or may use it.
The queried Apple API floor is iOS 8.0. With rustc 1.94.1, the Rust toolchain's minimum supported
iOS deployment target is 10.0; this package does not claim a linked Rust app can target iOS 8 or 9.

The status result describes only the calling app's Personal VPN connection at one query time.
`Invalid` is preserved as `NEVPNStatusInvalid`; it does not mean that no VPN is active device-wide.
A preference load error keeps its native error domain and integer code and is not mapped to a
disconnected status. Unknown future status values keep their raw integer.

The adapter only loads preferences and reads connection status or the two configuration flags. It
does not mutate flags, inspect Connect On Demand rules, save or remove a profile, start or stop a
tunnel, implement a provider, or report routes, reachability, tunnel health, or system-wide VPN
state. A false `isEnabled` value may reflect another enabled Personal VPN configuration, and
`isOnDemandEnabled` alone does not prove rule configuration or automatic connection behavior. The
preference-load completion runs on the caller's main thread. Callers must construct, poll, and drop
the returned future on that thread using `ios_runtime::main_thread::MainThread`; the native request
begins when the request function runs. Dropping the future detaches Rust interest but does not
cancel the native request.

See the [iOS status guide](../ios/vpn-status.md), [`framework-vpn`](../../crates/framework-vpn/README.md),
[`ios-vpn`](../../platform/ios/ios-vpn/README.md), and [implementation plan](../../PLAN_IOS_VPN_STATUS.md).
