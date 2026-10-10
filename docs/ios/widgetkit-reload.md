# iOS WidgetKit all-timeline reload request

`ios-widgetkit-reload::request_reload_all_timelines()` calls
`WidgetCenter.shared.reloadAllTimelines()` on iOS 14.0 and later. Its contract is a request for
WidgetKit to reload all configured widgets belonging to the containing app. `Ok(())` means the
synchronous request call returned; it does not mean a provider ran, a timeline was accepted, a view
was rendered, or any update occurred by a particular time

`ios-widgetkit-reload::invalidate_configuration_recommendations()` calls
`WidgetCenter.shared.invalidateConfigurationRecommendations()` when its iOS 16.0 symbol is
available. It invalidates and refreshes preconfigured intent configurations, but Apple documents
the method as inactive on iOS. `Ok(())` means only that the native method returned; it does not
promise that recommendations change or appear. On older iOS versions and non-iOS targets it returns
`WidgetKitError::NativeApiUnavailable`

Both operations are synchronous and make no main-thread or queue guarantee. The package does not
provide a widget extension, `TimelineProvider`, `Timeline`, SwiftUI `Widget`, widget configuration,
data sharing, or rendering. WidgetKit controls refresh budgets and scheduling. Neither operation
proves a configured widget exists. The local compiler and SDK evidence uses Xcode 26.6 / iOS SDK
26.5, below the repository's Xcode 27.x baseline

See the [B254/B435 WidgetKit plan record](../../PLAN_CAPABILITIES_WIDGETKIT.md) and the
[package README](../../platform/ios/ios-widgetkit-reload/README.md)

Primary API evidence: [WidgetCenter](https://developer.apple.com/documentation/widgetkit/widgetcenter),
[reloadAllTimelines()](https://developer.apple.com/documentation/widgetkit/widgetcenter/reloadalltimelines%28%29),
[invalidateConfigurationRecommendations()](https://developer.apple.com/documentation/widgetkit/widgetcenter/invalidateconfigurationrecommendations%28%29),
[Keeping a widget up to date](https://developer.apple.com/documentation/widgetkit/keeping-a-widget-up-to-date),
and [Timeline](https://developer.apple.com/documentation/widgetkit/timeline)

No tests, app launches, Simulator or device runs, WidgetCenter calls, provider callbacks, timeline
requests, or runtime probes were performed for B254
