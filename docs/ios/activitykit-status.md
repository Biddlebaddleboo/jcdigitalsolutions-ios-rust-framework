# iOS ActivityKit start-eligibility snapshot

`ios_activitykit_status::activities_enabled_for_current_app()` returns the current app's
`ActivityAuthorizationInfo.areActivitiesEnabled` value. Apple defines `true` as ActivityKit
reporting that the app can start a Live Activity now. The call is synchronous and read-only; it does
not create, enumerate, update, end, or observe Live Activities, and it does not present UI.

The result is not a promise that a later start will succeed. ActivityKit can reject a start for
other reasons, including device activity limits. This package does not provide `ActivityAttributes`,
Live Activity state, push tokens, App Intents, WidgetKit configuration, or SwiftUI presentation. A
host that offers Live Activities must set `NSSupportsLiveActivities` and implement a WidgetKit/
SwiftUI presentation; this package does not configure those host requirements.

The iOS API floor is 16.1. The SDK exposes this operation as Swift-only, so the package uses one
compiler-derived Swift-call C thunk for the exact initializer and getter and releases the returned
object through `swift-abi-core/apple-runtime`. No `.swift` source is shipped, and no Swift object
crosses the public Rust API. A missing weak-linked ActivityKit symbol returns
`ActivityAuthorizationError::NativeApiUnavailable`; no object returned from initialization maps to
`NativeValueUnavailable`.

Run `sh platform/ios/ios-activitykit-status/check.sh` for format, compile, strict Clippy, rustdoc,
compiler-ABI oracle, linked import inspection, and documentation checks. Its linked example is not
executed; the gate runs no tests or ActivityKit calls.

See the [B209 ActivityKit plan](../../PLAN_CAPABILITIES_ACTIVITYKIT.md) and
[`ios-activitykit-status`](../../platform/ios/ios-activitykit-status/README.md).
