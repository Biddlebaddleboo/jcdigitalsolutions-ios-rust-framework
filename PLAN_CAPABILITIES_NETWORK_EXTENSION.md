# PLAN_CAPABILITIES_NETWORK_EXTENSION.md — D82: NetworkExtension audit for row 098

## Scope

Audit row `098-extension-entitlement-capabilities-networkextension-vpn` for a public Rust-callable iOS surface, API floor, entitlements, distribution rules, and host lifecycle requirements

This is an evidence and scope recommendation only. It does not change the canonical matrix, global docs, CI, Cargo manifests, or source code

## Status and recommendation

### B79 implementation update

B79 now implements and validates the bounded Personal VPN profile-status query through the generated `objc2-network-extension` 0.3.2 binding. Root integrates row 098 as a portable-contract/iOS-backend `B` partial, adds `sh platform/ios/ios-vpn/check.sh` to macOS CI, and documents the exact entitlement and deployment limits. The broad NetworkExtension/VPN family remains unsupported beyond this read-only slice; see [B79](PLAN_IOS_VPN_STATUS.md).

At the D82 audit stage, row 098 was `X` because no bounded backend was implemented. The row covers a large framework: Personal VPN, custom packet and flow tunnel providers, content filters, DNS proxy/settings, app push, URL filters, and other platform-specific services. A query of `NEVPNManager` does not implement or represent that full capability

A credible future Rust slice is a read-only snapshot of the calling app's Personal VPN profile status through `NEVPNManager.sharedManager()`, `loadFromPreferencesWithCompletionHandler:`, and `connection.status`. This is a typed Objective-C API available to `objc2-network-extension` 0.3.2. The result could map the six public `NEVPNStatus` values into a portable enum and return a separate load error. It would not write preferences, start or stop a tunnel, inspect other apps' profiles, or claim the device's global VPN/routing state

The Personal VPN query was an API candidate at D82; B79 has since added `framework-vpn` and `ios-vpn` for the narrow status slice only

## SDK and API evidence

Inspection used Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5

- Framework path: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/NetworkExtension.framework`
- `NEVPNManager.h` declares `NEVPNManager`, `sharedManager`, `loadFromPreferencesWithCompletionHandler:`, and `connection` from iOS 8.0
- `NEVPNManager.sharedManager()` is the singleton for the calling process. `loadFromPreferencesWithCompletionHandler:` loads the caller's VPN preferences; Apple documents that its completion runs on the caller's main thread. If no configuration exists, the load completes with no error and an empty configuration
- `NEVPNConnection.status` and `NEVPNStatus` are available from iOS 8.0. The six SDK values are `Invalid`, `Disconnected`, `Connecting`, `Connected`, `Reasserting`, and `Disconnecting`. The header defines `Invalid` as not configured; Apple also documents that the connection status is `Invalid` if the configuration is absent
- `NEVPNManager` and `NEVPNConnection` instances are documented as thread-safe. The load callback's main-thread delivery remains a caller-visible callback-context constraint; it is not a reason to claim that the status getter is main-actor-only
- `NETunnelProviderManager` and `NETunnelProviderSession` begin at iOS 9.0. Their start methods return immediately after initiating connection; the immediate return is not proof that a tunnel reached `Connected`
- The minimum floor for the suggested Personal VPN status slice is iOS 8.0, not the iOS 9.0 floor of custom tunnel-provider APIs

Apple's Personal VPN docs describe built-in IPsec and IKEv2 configurations. A custom VPN protocol is a separate packet-tunnel-provider product and is not covered by this candidate

## Rust binding evidence and boundary

The generated-framework catalog in cached `objc2` 0.6.5 lists NetworkExtension's public Rust binding crate as `objc2-network-extension`. Its upstream 0.3.2 docs expose `NEVPNManager`, `NEVPNConnection`, and `NEVPNStatus`; the generated status is a transparent `NSInteger` wrapper with the six typed constants. D82 found the generated `loadFromPreferencesWithCompletionHandler` method under `block2`; B79 pins 0.3.2 with defaults disabled and only that NetworkExtension feature enabled

At D82 time the repository lock and package manifests did not include `objc2-network-extension`; B79 has since added it and passed the focused compile, strict Clippy, rustdoc, feature-tree, and link/import gates. The reviewed unsafe calls, object/block lifetimes, main-thread callback context, and error mapping are recorded in [B79](PLAN_IOS_VPN_STATUS.md)

An honest portable contract would report only the app-owned Personal VPN configuration status. `Invalid` must remain an explicit status value rather than a fabricated system-wide “VPN off” result. The async preference-load failure must remain distinct from a successful snapshot. The portable contract must not infer tunnel reachability, route coverage, provider health, VPN permission, or global device state from a profile status

## Entitlement, authorization, and distribution limits

- Apple documents the Personal VPN entitlement as `com.apple.developer.networking.vpn.api = ["allow-vpn"]`, enabled through the Personal VPN capability in Xcode. The entitlement page says this enables use of `NEVPNManager` to manage a Personal VPN configuration; a future package must not describe the query as entitlement-free
- Apple documents that a user must explicitly authorize the app the first time it saves a Personal VPN configuration. The proposed read-only path does not call save, remove, start, or stop APIs. The docs also say that loading before a first save returns an empty configuration. No interactive approval or creation flow belongs in this status-only slice
- The distinct `com.apple.developer.networking.networkextension` entitlement is an array of capability values such as `packet-tunnel-provider`, `app-proxy-provider`, `content-filter-provider`, `dns-proxy`, and `dns-settings`. It is not interchangeable with the Personal VPN entitlement; provider work needs the capability value specific to that provider and its own host setup
- Apple App Review Guideline 5.4 says apps offering VPN services must use `NEVPNManager` and may be offered only by developers enrolled as an organization. That rule is scoped to apps offering VPN services; this report does not extend it to every app that links NetworkExtension
- Apple's Network Extension provider deployment note lists extension packaging, OS floors, managed-device, supervised-device, and App Store restrictions by provider type. For example, iOS packet tunnel providers are app extensions from iOS 9.0, with per-app mode restricted to managed devices. A provider implementation requires a host app plus extension lifecycle and packaging, not merely an in-process Rust call
- Apple's developer agreement and entitlement approval remain host/distribution prerequisites. The chosen provider type and product model determine the applicable requirements; there is no single NetworkExtension entitlement or deployment rule that covers every API in the framework

## Candidate acceptance boundary

A follow-up may implement only an asynchronous, read-only Personal VPN status snapshot using the caller-process `NEVPNManager` singleton and iOS 8.0 APIs. It may represent load failure plus the six public statuses. It must preserve the documented main-thread completion context, make no preference mutation or tunnel-control call, and state that the status is for this app's Personal VPN profile

That candidate must not claim:

- all VPNs or the active system route are represented by the app's `NEVPNManager` connection
- profile status proves network reachability, successful VPN service, or provider health
- a successful load means the user granted VPN authorization or that a configuration exists
- an immediate tunnel-start result means the connection is established
- generic NetworkExtension, packet tunnel, content filter, DNS, or app proxy support
- entitlement approval, App Store eligibility, or host configuration from a successful Rust compile/link

## Apple and binding references

- [NetworkExtension framework](https://developer.apple.com/documentation/networkextension)
- [NEVPNManager](https://developer.apple.com/documentation/networkextension/nevpnmanager)
- [Personal VPN](https://developer.apple.com/documentation/networkextension/personal-vpn)
- [NEVPNManager loadFromPreferencesWithCompletionHandler:](https://developer.apple.com/documentation/networkextension/nevpnmanager/loadfrompreferences%28completionhandler%3A%29?language=objc)
- [NEVPNManager connection](https://developer.apple.com/documentation/networkextension/nevpnmanager/connection)
- [NEVPNConnection status](https://developer.apple.com/documentation/networkextension/nevpnconnection/status)
- [Personal VPN Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.vpn.api)
- [Network Extensions Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.networking.networkextension)
- [Configuring Network Extensions](https://developer.apple.com/documentation/xcode/configuring-network-extensions)
- [TN3134: Network Extension provider deployment](https://developer.apple.com/documentation/technotes/tn3134-network-extension-provider-deployment)
- [App Review Guidelines 5.4: VPN Apps](https://developer.apple.com/app-store/review/guidelines/uk/)
- [Apple Developer Program agreements and guidelines](https://developer.apple.com/support/terms/)
- [`objc2-network-extension` 0.3.2 `NEVPNManager`](https://docs.rs/objc2-network-extension/0.3.2/objc2_network_extension/struct.NEVPNManager.html)
- [`objc2-network-extension` 0.3.2 `NEVPNConnection`](https://docs.rs/objc2-network-extension/0.3.2/objc2_network_extension/struct.NEVPNConnection.html)
- [`objc2-network-extension` 0.3.2 `NEVPNStatus`](https://docs.rs/objc2-network-extension/0.3.2/objc2_network_extension/struct.NEVPNStatus.html)
- [`objc2-network-extension` 0.3.2 feature set](https://docs.rs/crate/objc2-network-extension/0.3.2/features)
- Local SDK headers: `NetworkExtension.framework/Headers/NEVPNManager.h`, `NEVPNConnection.h`, `NETunnelProviderManager.h`, and `NETunnelProviderSession.h` under the framework path above
- Local binding catalog: `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_data.md`
- Current row record: `docs/capabilities/capability-status.json`, id `098-extension-entitlement-capabilities-networkextension-vpn`

## Audit record

Inspected the installed NetworkExtension headers and framework module, the generated-framework catalog, the repository lock and package manifests, row 098's current status record, upstream `objc2-network-extension` 0.3.2 API/feature docs, and Apple primary API, entitlement, deployment, and review documentation

No source, build, link, test, device, Simulator, entitlement, or runtime probe was run for D82. B79 later added the bounded implementation and passed compile/link gates; its link probes were inspected but not executed. Root marks row 098 `B`/partial for the Personal VPN profile-status snapshot only; no entitlement access or runtime VPN behavior was exercised
