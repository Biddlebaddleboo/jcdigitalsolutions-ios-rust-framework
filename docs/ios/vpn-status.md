# iOS Personal VPN status and configuration queries

`ios-vpn::request_personal_vpn_status(main_thread)` starts one read of the calling app's Personal
VPN preferences and returns `IosPersonalVpnStatusFuture`. The caller must obtain
`ios_runtime::main_thread::MainThread::current()` on the main thread and keep construction, polling,
and drop of the future on that thread. The future is `!Send` and `!Sync`. Apple documents that
`loadFromPreferencesWithCompletionHandler:` calls its completion on the caller's main thread.

On successful preference load, the status callback reads `NEVPNManager.sharedManager().connection.status`
and maps `Invalid`, `Disconnected`, `Connecting`, `Connected`, `Reasserting`, and `Disconnecting` to
`PersonalVpnStatus`. Any raw status value not recognized by this crate is preserved as
`PersonalVpnStatus::Unknown(isize)`. A failed load returns
`PersonalVpnQueryError::PreferenceLoad(PersonalVpnLoadError)`, whose domain and native integer code
are owned Rust values. Callback-thread mismatch and a Rust panic during callback processing remain
separate query errors.

`ios-vpn::request_personal_vpn_configuration(main_thread)` starts a separate preference load and
returns `IosPersonalVpnConfigurationFuture`; its `Future` output is
`Result<PersonalVpnConfigurationFlags, PersonalVpnQueryError>`. The owned result contains the
loaded configuration's `enabled` and `on_demand_enabled` booleans, read from
`NEVPNManager.isEnabled` and `NEVPNManager.isOnDemandEnabled`. Apple notes that another enabled
Personal VPN configuration can make `isEnabled` false and that preferences must be loaded again to
observe that change. The Connect On Demand flag does not establish that rules exist or will trigger
a connection. This query performs no setters, rule reads, profile writes, or tunnel control; it
retains the same main-thread, entitlement, future-drop, and preference-load-error contract as the
status query.

Each request starts when its function runs rather than when the future is first polled. Dropping a
future abandons the Rust result and waker; it does not cancel Apple's preference-load operation.
The status query reads no profile fields; the configuration query reads only the two enabled flags
and does not mutate profile fields or issue tunnel control. `Invalid` is the native status for an
invalid or unconfigured profile, not a device-wide VPN-off result. These snapshots say nothing
about routes, network reachability, tunnel health, other apps' configurations, or all VPN
implementations on the device.

The host app must have the Personal VPN entitlement
`com.apple.developer.networking.vpn.api = ["allow-vpn"]`, subject to Apple's provisioning and
distribution requirements. Neither compilation nor link inspection verifies entitlement access
or runtime behavior. The API floor for the queried NetworkExtension methods is iOS 8.0. With rustc
1.94.1, the minimum supported Rust app deployment target is iOS 10.0, so this package does not
claim a linked Rust app can target iOS 8 or 9.

The generated `objc2-network-extension` 0.3.2 crate is used with default features disabled and its
`block2` feature enabled. The linked probe checks only the compiled framework imports and selectors;
it is not executed and does not inspect device or Simulator VPN state.

See the [capability guide](../capabilities/vpn.md), [implementation plan](../../PLAN_IOS_VPN_STATUS.md),
and [validation commands](../../PLAN_IOS_VPN_STATUS.md#validation-gates).
