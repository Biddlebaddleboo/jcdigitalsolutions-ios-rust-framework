# `framework-vpn`

This `no_std` crate defines owned values for the calling app's Personal VPN status and
configuration-flag queries. It maps the six public `NEVPNStatus` values and preserves an unknown
raw integer. `PersonalVpnConfigurationFlags` holds only the loaded configuration's `enabled` and
`on_demand_enabled` booleans. A preference-load error owns its native error domain and `NSInteger`
code. It does not mutate configuration and does not report device-wide VPN state, entitlement
approval, tunnel health, Connect On Demand rule behavior, or network routes.

The iOS adapter is [`ios-vpn`](../../../platform/ios/ios-vpn/README.md).
