# `ios-vpn`

This iOS-only package implements `request_personal_vpn_status(MainThread)` and
`request_personal_vpn_configuration(MainThread)` for the calling app's Personal VPN profile. Each
starts a preference load immediately and returns a main-thread-bound future. After a successful
load, the status callback reads only the connection's `NEVPNStatus`; the configuration callback
reads only `NEVPNManager.isEnabled` and `isOnDemandEnabled`. Both copy the native preference-load
error domain and `NSInteger` code into Rust-owned data.

The host app must carry the Personal VPN entitlement
`com.apple.developer.networking.vpn.api = ["allow-vpn"]`, subject to Apple's approval. The API
floor is iOS 8.0, while rustc 1.94.1 sets iOS 10.0 as its minimum supported deployment target. The
configuration query does not read Connect On Demand rules or change either flag. A false `enabled`
value can mean another Personal VPN configuration is enabled; `on_demand_enabled` does not
establish that rules exist or run. Neither query starts or stops a tunnel or reports global VPN
state, routes, reachability, or tunnel health. Dropping either future does not cancel its native
request.

The NetworkExtension binding is `objc2-network-extension` 0.3.2 with default features disabled and
only its `block2` feature enabled. No Objective-C ABI is handwritten. The package-scoped gate is
`sh platform/ios/ios-vpn/check.sh`; it does not run tests or execute its linked probes.

See the [capability guide](../../../docs/capabilities/vpn.md), [iOS guide](../../../docs/ios/vpn-status.md),
and [implementation plan](../../../PLAN_IOS_VPN_STATUS.md).
