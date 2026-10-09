# `ios-vpn`

This iOS-only package implements `request_personal_vpn_status(MainThread)` for the calling app's
Personal VPN profile. It starts a preference load immediately and returns a main-thread-bound
future. The callback reads only the connection's `NEVPNStatus` after a successful load. It copies
the native preference-load error domain and `NSInteger` code into Rust-owned data.

The host app must carry the Personal VPN entitlement
`com.apple.developer.networking.vpn.api = ["allow-vpn"]`, subject to Apple's approval. The API
floor is iOS 8.0, while rustc 1.94.1 sets iOS 10.0 as its minimum supported deployment target. The
adapter does not inspect or change VPN configuration, start or stop a tunnel,
or report global VPN state, routes, reachability, or tunnel health. Dropping the future does not
cancel the native request.

The NetworkExtension binding is `objc2-network-extension` 0.3.2 with default features disabled and
only its `block2` feature enabled. No Objective-C ABI is handwritten. The package-scoped gate is
`sh platform/ios/ios-vpn/check.sh`; it does not run tests or execute its linked probes.

See the [capability guide](../../../docs/capabilities/vpn.md), [iOS guide](../../../docs/ios/vpn-status.md),
and [B79 plan](../../../PLAN_IOS_VPN_STATUS.md).
