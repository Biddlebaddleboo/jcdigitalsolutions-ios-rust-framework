# `ios-activitykit-status`

This iOS-only package exposes `activities_enabled_for_current_app()` as one synchronous snapshot
of `ActivityAuthorizationInfo.areActivitiesEnabled`. Apple defines the Boolean as whether this app
can start a Live Activity now. It does not create, enumerate, update, end, or observe activities,
or claim that a later start will succeed.

The API is Swift-only in the SDK. The package uses a compiler-derived, capability-specific C
`swiftcall` thunk and the audited `swift-abi-core/apple-runtime` feature to release the owned
`ActivityAuthorizationInfo` object. It includes no `.swift` source and exposes no reusable Swift ABI.
The API floor is iOS 16.1; the weak-imported Swift symbols yield `NativeApiUnavailable` if absent.

For an app that offers Live Activities, Apple requires the host to enable
`NSSupportsLiveActivities` and provide a WidgetKit/SwiftUI presentation. This package does not
configure that host behavior, request permission, manage ActivityKit state, provide presentation,
or implement App Intents or push updates. `Ok(true)` is not a guarantee that a later start succeeds.

`sh platform/ios/ios-activitykit-status/check.sh` runs static compilation, compiler-ABI comparison,
link/import inspection, and docs checks. The link examples are built and inspected, never executed;
the gate runs no tests or ActivityKit calls.

See the [ActivityKit feasibility and B209 plan](../../../PLAN_CAPABILITIES_ACTIVITYKIT.md) and
[iOS guide](../../../docs/ios/activitykit-status.md).
