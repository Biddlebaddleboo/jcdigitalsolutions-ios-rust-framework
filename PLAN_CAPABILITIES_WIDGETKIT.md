# PLAN_CAPABILITIES_WIDGETKIT.md — D97: WidgetKit Feasibility for Row 111

## Objective

Audit whether row `111-compiler-build-host-capabilities-widgetkit-support-management-data-logic-available-through-proven-interfaces-do-not-implement-a-swiftui-clone-merely-to-claim-full-rendering-support` has a small WidgetKit management, data, or status surface that Rust can call through a proven public route. This report does not change the canonical row, add code, or repeat C7's App Intents metadata audit

## Status and recommendation

B254 makes row 111 partial (`B`) for `WidgetCenter.shared.reloadAllTimelines()` only. This synchronous request operates on configured widgets belonging to the containing app. It does not guarantee provider success, rendering, or timing. The API is Swift-only; B254 uses a compiler-derived weak `swiftcall` bridge rather than a C/Objective-C declaration

B254 implements the request-only timeline reload. Reading user-configured widget descriptors remains a separate candidate requiring Swift class/closure/Result/WidgetInfo ownership support. It would not implement a widget provider, render content, prove that a widget exists in an extension, or guarantee when WidgetKit refreshes its view

Full widget support remains outside a small Rust facade. `TimelineProvider` has a Swift associated `Entry` type and Swift protocol requirements that return or callback with `Timeline<Entry>`; `Timeline` is generic over a `TimelineEntry`. A WidgetKit extension also defines a SwiftUI `Widget` whose body returns `WidgetConfiguration`; its configuration initializers take `@ViewBuilder` closures with `Content: View`. These APIs need Swift protocol conformance, associated-type metadata, callback/async interop, and SwiftUI view construction. This report does not propose a SwiftUI clone

## Scope distinct from C7

C7 (`PLAN_SWIFT_APP_INTENTS.md` and `docs/swift-abi/APP_INTENTS_STAGE0.md`) audits App Intents metadata inputs and `AppIntent.perform()` runtime entry. D97 instead audits `WidgetCenter`, widget configuration lookup, timeline/provider protocols, and the SwiftUI view boundary. `AppIntentTimelineProvider` is noted only as one provider variant; its App Intents metadata and async runtime limits remain with C7 and C6

## Installed SDK and binding evidence

Audit host: Xcode 26.6, build 17F113; iPhoneOS SDK 26.5, path `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk`

- Public umbrella header: `System/Library/Frameworks/WidgetKit.framework/Headers/WidgetKit.h`; it imports `WGWidgetDefines.h`
- `WGWidgetDefines.h` declares `WGWidgetUserInfoKeyKind`, `WGWidgetUserInfoKeyFamily`, `WGWidgetUserInfoKeyActivityID`, and `NSUserActivityTypeLiveActivity`. It declares no `WidgetCenter`, provider, timeline, or widget-configuration C/Objective-C methods
- `WidgetKit.swiftmodule/arm64e-apple-ios.swiftinterface` declares `WidgetCenter`, `WidgetInfo`, `TimelineEntry`, `TimelineProvider`, `Timeline`, `IntentTimelineProvider`, and `AppIntentTimelineProvider` as Swift APIs
- `WidgetKit.tbd` exports Swift-mangled symbols for `WidgetCenter`, `WidgetInfo`, and provider protocol requirements. Those exports are evidence of a Swift ABI, not a stable C/Objective-C entry point or a Rust binding contract
- A local Cargo registry search found no `objc2-widget-kit` or other WidgetKit binding. A source search found no WidgetKit Rust adapter in the repo
- `PLAN_SWIFT_ABI.md` says C1–C6 are bounded ABI proofs, not a general production Swift value/async adapter; C6 found no documented public task-entry/context/resume contract. Thus this audit cannot treat a Swift symbol export as proof that a Rust caller can safely invoke WidgetKit

Relevant local declarations:

- `WidgetKit.framework/Modules/WidgetKit.swiftmodule/arm64e-apple-ios.swiftinterface`, `WidgetCenter` and `WidgetInfo` declarations: iOS 14.0 baseline
- Same interface, `TimelineEntry`, `TimelineProvider`, and `Timeline`: iOS 14.0 baseline
- Same interface, `AppIntentTimelineProvider`: iOS 17.0; its `snapshot` and `timeline` requirements are `async`
- `SwiftUI.framework/Modules/SwiftUI.swiftmodule/arm64e-apple-ios.swiftinterface`, `Widget` and `WidgetConfiguration`: iOS 14.0 and `@MainActor`; their associated `Body` and `body` properties use SwiftUI configuration/view values

## Public surface and API floors

| Surface | iOS floor in audited interface | Meaning and Rust boundary |
| --- | ---: | --- |
| `WidgetCenter.shared`, `reloadTimelines(ofKind:)`, `reloadAllTimelines()` | 14.0 | Swift class property/methods; `reload...` asks WidgetKit for new timeline work, not a promise of immediate or exact-time render |
| `WidgetCenter.getCurrentConfigurations(_:)` | 14.0 | Swift `@Sendable` closure receives `Result<[WidgetInfo], Error>`; `WidgetInfo` contains `kind`, `family`, and optional legacy `INIntent` configuration |
| `WidgetCenter.invalidateConfigurationRecommendations()` | 16.0 | Swift method for preconfigured recommendations, not widget render or provider status |
| `WidgetCenter.currentConfigurations() async throws` | 18.0 | Async Swift method; C6 has no supported public task-entry/context/resume route for a Rust async caller |
| `TimelineEntry`, `TimelineProvider`, `Timeline<EntryType>` | 14.0 | Swift protocol with associated entry type; `Timeline` is generic; provider methods use callback closures and a `Timeline<Entry>` value |
| `IntentTimelineProvider` | 14.0 | Swift protocol with associated `Entry` and `Intent`, plus callbacks; its newer async relevance requirement is iOS 18.0 |
| `AppIntentTimelineProvider` | 17.0 | Swift protocol with associated `Entry` and `Intent`; `snapshot` and `timeline` are async, so C6 applies; App Intents metadata stays in C7 |
| SwiftUI `Widget` and `WidgetConfiguration` | 14.0 | `@MainActor` protocols with associated `Body`; WidgetKit configuration initializers take SwiftUI `ViewBuilder` closures |

The manifest leaves `minimum_ios_version`, `required_frameworks`, `required_permissions`, `required_info_plist_keys`, and `entitlements` null, and labels metadata unverified. This audit does not convert those nulls into a claim that no host configuration is needed

## Host, update, and data limits

- Apple directs apps to add a Widget Extension target; widget content uses SwiftUI. Shared app/widget data may use an App Group container, but this report does not mark an App Group as required for the `WidgetCenter` calls
- `getCurrentConfigurations(_:)` reports user-configured widget descriptors. It is not a general support query and does not prove provider health, extension registration, a successful timeline callback, or a rendered view
- `WidgetCenter` has no `@MainActor` annotation in the audited interface; `getCurrentConfigurations(_:)` marks its callback `@Sendable`. The provider protocol has no main-actor annotation there, while SwiftUI `Widget` and `WidgetConfiguration` are `@MainActor`. Do not infer a broader thread guarantee from the absent annotation
- `reloadTimelines(ofKind:)` and `reloadAllTimelines()` request timeline reloads. Apple says widgets have limited daily refreshes, and a `Timeline` policy is the earliest time WidgetKit requests a new timeline; the view may update later than an entry date
- The provider path is an extension callback surface. WidgetKit may request snapshots or timelines at system-selected times; a provider must return `TimelineEntry`/`Timeline` data. The entry date or reload request is not a delivery-time guarantee
- No prompt or privacy permission is established for the management calls. This audit does not verify all signing, distribution, extension, or capability requirements for a shipped widget
- Do not claim Live Activity behavior, controls, App Intents actions, push reloads, widget rendering, or app-group data exchange as implemented by a management-only facade

## Acceptance boundary

This audit establishes only:

- A possible future Swift ABI slice for `WidgetCenter` management and configured-widget descriptors, subject to a supported Rust call path
- The iOS 14.0 floor for the base management/configuration/provider APIs; iOS 16.0 for `invalidateConfigurationRecommendations`; iOS 17.0 for `AppIntentTimelineProvider`; and iOS 18.0 for `currentConfigurations()` and the newer async provider requirements
- The current reason row 111 remains unsupported: no Rust-callable WidgetKit adapter is present, and the provider/render path needs Swift protocol and SwiftUI types

It does not establish a package, binding feature, entitlement, usage-description key, successful API call, configured widget, extension lifecycle, timeline execution, update schedule, or rendered output

## Apple primary sources

- [WidgetKit](https://developer.apple.com/documentation/widgetkit/)
- [WidgetCenter](https://developer.apple.com/documentation/widgetkit/widgetcenter)
- [WidgetCenter.reloadTimelines(ofKind:)](https://developer.apple.com/documentation/widgetkit/widgetcenter/reloadtimelines%28ofkind%3A%29)
- [WidgetCenter.reloadAllTimelines()](https://developer.apple.com/documentation/widgetkit/widgetcenter/reloadalltimelines%28%29)
- [WidgetCenter.getCurrentConfigurations(_:)](https://developer.apple.com/documentation/widgetkit/widgetcenter/getcurrentconfigurations%28_%3A%29)
- [WidgetCenter.currentConfigurations()](https://developer.apple.com/documentation/widgetkit/widgetcenter/currentconfigurations%28%29)
- [TimelineProvider](https://developer.apple.com/documentation/widgetkit/timelineprovider)
- [Timeline](https://developer.apple.com/documentation/widgetkit/timeline)
- [Creating a widget extension](https://developer.apple.com/documentation/widgetkit/creating-a-widget-extension)
- [Developing a WidgetKit strategy](https://developer.apple.com/documentation/widgetkit/developing-a-widgetkit-strategy)
- [SwiftUI Widget](https://developer.apple.com/documentation/swiftui/widget)

## Root integration need

No Cargo, lock, CI, source, matrix, aggregate-plan, or docs-index change is needed for this feasibility report. If root accepts the recommendation, it may revise only row 111's status reason to distinguish Swift-only `WidgetCenter` management from the absent provider/render path; keep the row `X` until a supported Rust call adapter exists

## B254 implementation — all-configured-widget timeline reload request

B254 adds `ios-widgetkit-reload::request_reload_all_timelines()` for iOS 14.0 and later. Its exact
native operation is `WidgetCenter.shared.reloadAllTimelines()`, which Apple documents as a request
to reload timelines for all configured widgets belonging to the containing app. The method is
synchronous and returns `Void`; `Ok(())` means only that this native request call returned. It
does not prove a widget exists, a provider ran, a new timeline was accepted, a widget rendered, or
an update occurred by a particular time

The iOS 26.5 public Swift interface declares `WidgetCenter.shared` and `reloadAllTimelines()` from
iOS 14.0. The public WidgetKit C/Objective-C headers declare no `WidgetCenter` type or method. The
device compiler oracle lowers metadata accessor → owned `shared` getter → zero-argument `Void`
method → `swift_release`; the device and Simulator call patterns match compiler-derived
`swiftcall` thunks. The bridge uses weak imports and `swift-abi-core/apple-runtime` to retain the
manager through the call and release it afterward. It adds no Swift source, selector, generated
binding, callback, closure, or async runtime route

This is a host-owned management operation, not a general WidgetKit availability query. The
containing app should request reloads when its widget data changes and should respect WidgetKit's
dynamic per-widget refresh budget. WidgetKit controls provider scheduling and render timing; the
package makes no main-thread or queue guarantee and does not implement an extension, provider,
timeline, data-sharing contract, SwiftUI view, Live Activity, or Control

Capability-row impact: row
`111-compiler-build-host-capabilities-widgetkit-support-management-data-logic-available-through-proven-interfaces-do-not-implement-a-swiftui-clone-merely-to-claim-full-rendering-support`
can move from `X` to partial (`B`) for this all-configured-widget reload request only. Provider
execution, configured-widget inventory, data logic, configuration, rendering, and refresh timing
remain outside the implemented contract. Root owns any aggregate matrix edit

The focused B254 gate runs package-only format, host/device/Simulator `cargo check`, strict Clippy,
rustdoc, `docs-check`, compiler-oracle checks for device and Simulator, and static WidgetKit
framework link/import checks. It runs no tests, app, Simulator or device calls, provider callbacks,
timeline requests, or runtime probes. The local compiler is Xcode 26.6 / iOS SDK 26.5, below the
repository's Xcode 27.x baseline

Changed paths: `platform/ios/ios-widgetkit-reload/`, `docs/ios/widgetkit-reload.md`, and this
focused plan only. Root aggregate plans and capability matrix remain unchanged
