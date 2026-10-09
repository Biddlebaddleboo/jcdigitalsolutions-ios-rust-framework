# iOS DockKit system-tracking setting snapshot

`ios-dockkit-status` reads `DockAccessoryManager.shared.isSystemTrackingEnabled` on physical iOS
devices through compiler-derived C `swiftcall` thunks and the audited
`swift-abi-core/apple-runtime` path for the owned manager object

The Boolean reports only whether system tracking is enabled. It does not establish accessory
presence, active tracking, camera access, or support for a requested operation. The call does not
change the setting, read camera frames, or observe dock/undock events

The API floor is iOS 17.0. The installed iOS Simulator SDK has no DockKit framework; Simulator and
non-iOS targets return `NativeApiUnavailable`. The public Swift interface has no `@MainActor`
annotation, but this package makes no main-thread or queue guarantee. The local checks use Xcode
26.6 / iOS 26.5, below the repository's Xcode 27.x baseline

See the [B245 DockKit plan record](../../PLAN_CAPABILITIES_DOCKKIT.md) and the
[package README](../../platform/ios/ios-dockkit-status/README.md)
