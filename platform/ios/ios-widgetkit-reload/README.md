# iOS WidgetKit management requests

`ios-widgetkit-reload` exposes two synchronous, request-only operations:

- `request_reload_all_timelines()` calls `WidgetCenter.shared.reloadAllTimelines()` on iOS 14.0 and
  later. It requests a reload for configured widgets belonging to the containing app; it does not
  guarantee provider invocation, rendering, or an update time
- `invalidate_configuration_recommendations()` calls
  `WidgetCenter.shared.invalidateConfigurationRecommendations()` when its iOS 16.0 symbol is
  available. Apple documents the method as inactive on iOS, so a successful return does not promise
  that recommendations change or appear

Both calls use compiler-derived C `swiftcall` thunks and the audited `swift-abi-core/apple-runtime`
path for the owned `WidgetCenter.shared` object. The iOS 16.0 method is weakly imported; unavailable
symbols return `WidgetKitError::NativeApiUnavailable`. Non-iOS targets return that error for both
operations. The package adds no Swift source, Objective-C selector, or generated binding. Neither
operation makes a main-thread claim or implements a widget provider, extension, timeline, or view

See the [B254/B435 WidgetKit plan record](../../../PLAN_CAPABILITIES_WIDGETKIT.md) and the
[capability guide](../../../docs/ios/widgetkit-reload.md)
