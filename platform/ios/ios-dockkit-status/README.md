# iOS DockKit system-tracking setting snapshot

This crate exposes only `DockAccessoryManager.shared.isSystemTrackingEnabled` on physical iOS
devices through compiler-derived C `swiftcall` thunks

The value reports the system-tracking setting only. It does not show accessory presence, active
tracking, camera access, or operation success. It does not change the setting, read camera frames,
or observe DockKit events

The API floor is iOS 17.0. The installed Simulator SDK has no DockKit framework, so Simulator and
non-iOS targets return `DockKitError::NativeApiUnavailable`. The crate uses `swift-abi-core` for
the owned manager object and adds no Swift source

See [the focused guide](../../../docs/ios/dockkit-status.md) and
[B245 plan record](../../../PLAN_CAPABILITIES_DOCKKIT.md)
