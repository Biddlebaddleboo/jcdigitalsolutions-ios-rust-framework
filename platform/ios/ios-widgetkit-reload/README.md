# iOS WidgetKit all-timeline reload request

`ios-widgetkit-reload` exposes `request_reload_all_timelines()`, which calls
`WidgetCenter.shared.reloadAllTimelines()` on iOS 14.0 and later. It requests a reload for every
configured widget belonging to the containing app; it does not guarantee provider invocation,
rendering, or an update time

The package uses compiler-derived C `swiftcall` thunks and the audited
`swift-abi-core/apple-runtime` path for the owned `WidgetCenter.shared` object. It adds no Swift
source, Objective-C selector, or generated binding. Non-iOS targets return
`WidgetKitReloadError::NativeApiUnavailable`

See the [B254 WidgetKit plan record](../../../PLAN_CAPABILITIES_WIDGETKIT.md) and the
[capability guide](../../../docs/ios/widgetkit-reload.md)
