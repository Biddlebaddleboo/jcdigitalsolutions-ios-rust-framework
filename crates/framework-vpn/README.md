# `framework-vpn`

This `no_std` crate defines owned values for the calling app's Personal VPN status query. It maps
the six public `NEVPNStatus` values and preserves an unknown raw integer. A preference-load error
owns its native error domain and `NSInteger` code. It does not read or write configuration and
does not report device-wide VPN state, entitlement approval, tunnel health, or network routes.

The iOS adapter is [`ios-vpn`](../../../platform/ios/ios-vpn/README.md).
