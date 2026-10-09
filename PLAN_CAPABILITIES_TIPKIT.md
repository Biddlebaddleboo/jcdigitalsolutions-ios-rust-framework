# PLAN_CAPABILITIES_TIPKIT.md — D73: TipKit status/value feasibility

## Objective

Audit row `092-commerce-services-tipkit-only-if-system-tipkit-behavior-is-specifically-requested-otherwise-framework-owned-tip-logic-may-be-portable` for a narrow system-tip status or value API that Rust can call without implementing presentation or inventing SwiftUI behavior. This is a feasibility audit only

## Status

TipKit has a non-presentation status value, but it is per concrete Swift `Tip` value and depends on that value's Swift rules/options plus TipKit configuration and persisted state. The `Tip` protocol, its `status`/`shouldDisplay` getters, and `Tips.configure` are Swift-only. No generated Rust binding or C query exists. Keep row 092 unsupported (`X`) for Apple TipKit behavior; a Rust-native hint engine remains a distinct portable implementation, not TipKit parity

## Smallest status candidate and blocker

The narrowest semantic value is `Tip.status` (or its Boolean projection `Tip.shouldDisplay`) for one host-defined `Tip` type. On iOS 17.0+, Apple defines `status` as current display eligibility based on that tip's rules and configured display frequency; `Tips.Status` cases are `pending`, `available`, and `invalidated(reason)`. `shouldDisplay` is true only when status is `available`. This is an eligibility snapshot, not evidence that a tip view is mounted, visible, or presented

This is not a global system-tip status query. The `Tip` protocol requires a host-defined type with SwiftUI `Text`/`Image` values, rules, actions, and options. Rules can depend on `Tips.Parameter`, `Tips.Event`, and Swift rule-builder/macro values. `statusUpdates` and `shouldDisplayUpdates` are Swift `AsyncStream` types. Rust cannot construct or query an arbitrary conforming tip through the installed Objective-C bindings or a public C interface

The future narrow route would require a specifically approved Swift-to-C wrapper for one fixed tip type, with explicit scalar status mapping and host-owned `Tips.configure` lifecycle. That would add Swift/wrapper ABI requirements and would not generalize to arbitrary TipKit tips. D73 does not add that wrapper

## SDK and binding evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, Swift interface compiler 6.3.2, and Rust/Cargo 1.94.1

- The iPhoneOS SDK's `TipKit.framework` provides a public `TipKit.swiftinterface`, `.swiftdoc`, and `TipKit.tbd`; it has no public framework C headers or module map
- `Tip` is a public Swift protocol available from iOS 17.0. Its properties include `title: SwiftUICore.Text`, `message: SwiftUICore.Text?`, `image: SwiftUICore.Image?`, and builder-based actions, rules, and options. The extension exposes `status`, `shouldDisplay`, and async status streams; these declarations are not `@objc`
- `Tips.configure(_:)` is a synchronous Swift static method that throws. Apple requires configuration before tips display; the configuration loads persistent tip state. The default datastore is local to the app, with an immediate default display-frequency option
- `Tip.Status`/`Tips.Status` is a Swift enum available from iOS 17.0. Its payload-bearing `invalidated(reason)` case and its `Hashable`/`Sendable` conformances do not make it a C enum or Objective-C value
- `TipUIView`, `TipUICollectionViewCell`, and `TipUIPopoverViewController` are UIKit presentation classes. `TipUIView` is `@objc` and `@MainActor`, but its public initializer accepts `any Tip` and an actor-isolated action closure; the initializer and TipKit status APIs are Swift declarations, not an Objective-C status interface. It is a presentation view, not a status query
- The SDK TAPI lists Objective-C runtime class `_TtC6TipKit9TipUIView`, but that does not expose Tip protocol values or status as C/Objective-C APIs
- The installed `objc2` 0.6.5 generated-framework inventory marks `TipKit` Swift-only. The local Cargo registry contains no `objc2-tipkit` crate or generated TipKit binding
- The TipKit status declarations have no `@MainActor` annotation in the Swift interface. The UIKit tip views are `@MainActor`; Apple does not document a separate queue/thread-safety contract for status reads. Do not infer arbitrary-thread safety from the missing actor annotation
- The public API floor for `Tip`, status, and `TipUIView` is iOS 17.0. New style properties in the inspected SDK start at iOS 18.0 or iOS 26.0 and are not required for a status-only value

## UI side effects, persistence, and host configuration

- Reading a tip's status does not itself display a view. `TipView`, `popoverTip`, and `TipUIView` are presentation APIs; the UIKit/SwiftUI views and their action handlers are excluded
- Apple says `Tips.configure(_:)` must be called before tips display and loads/configures persistent state. The default location is generally the app's support directory; default TipKit state does not sync through CloudKit
- `Tips.Parameter` values are persistent by default unless configured as transient. Rule/event state therefore participates in TipKit's datastore behavior; do not expose or claim persistence semantics from a Rust facade without modeling that state contract
- No permission prompt, usage-description key, or entitlement is documented for the default local TipKit store. Optional CloudKit synchronization requires iCloud and Background Modes capabilities. An optional group-container datastore must use an identifier present in the app's App Groups entitlement
- TipKit UI displays contextual application guidance, not system authorization UI. Do not present tips or trigger actions as part of a status/value API

## Recommendation and next step

No narrowly scoped system TipKit status/value API is safely callable from Rust using the current public C/Objective-C surface and cached bindings. Keep row 092 at `X`. If system TipKit interoperability is specifically required, create a separate Swift-ABI/C-wrapper feasibility task for one fixed tip type and scalar `Tip.Status` mapping; otherwise use a portable Rust-owned eligibility model and label it as non-TipKit behavior

## Apple primary documentation

- [TipKit framework](https://developer.apple.com/documentation/tipkit)
- [Tip protocol](https://developer.apple.com/documentation/tipkit/tip)
- [Tip status](https://developer.apple.com/documentation/tipkit/tip/status-swift.property)
- [Tip shouldDisplay](https://developer.apple.com/documentation/tipkit/tip/shoulddisplay)
- [Tips.Status.available](https://developer.apple.com/documentation/tipkit/tips/status/available)
- [Tips.Status.pending](https://developer.apple.com/documentation/tipkit/tips/status/pending)
- [Tips.configure(_:)](https://developer.apple.com/documentation/tipkit/tips/configure(_:))
- [TipUIView](https://developer.apple.com/documentation/tipkit/tipuiview)
- [TipKit datastore location](https://developer.apple.com/documentation/tipkit/tips/configurationoption/datastorelocation)
- [TipKit group-container datastore](https://developer.apple.com/documentation/tipkit/tips/configurationoption/datastorelocation/groupcontainer(identifier:))
- [TipKit CloudKit configuration](https://developer.apple.com/documentation/tipkit/tips/configurationoption/cloudkitcontainer(_:))

No tests, builds, link probes, datastore mutation, or Tip presentation were performed

## B191 follow-up: no C/Objective-C TipKit status surface

Rechecked row 092 against the installed iOS 26.5 interface and current Apple docs. The useful
scalar candidate is still `Tip.status` or its Boolean projection `Tip.shouldDisplay`: Apple defines
it as this concrete tip's current eligibility based on its rules and configured display frequency.
The getter belongs to the Swift `Tip` protocol extension; `Tip` itself requires host-defined
`Text`, `Image`, rule, action, and option values. Its status and updates are not Objective-C
declarations, and there is no public TipKit header/module map or generated Rust binding in the
installed SDK/catalog.

The apparent Objective-C-adjacent `TipUIView` does not supply a status shortcut. Apple documents
it as an `@MainActor` UIKit presentation view whose initializer takes `any Tip` and an action
closure; this is a display API and still depends on the Swift tip value. `Tips.configure(_:)` is
also Swift-only and loads persistent TipKit state before tips display. Neither provides a
stateless, global framework-ready query.

The installed declaration is
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/TipKit.framework/Modules/TipKit.swiftmodule/arm64e-apple-ios.swiftinterface`;
it marks `Tip` and `status` available from iOS 17.0, without `@objc`. Apple's status docs confirm
the per-tip eligibility meaning, not app-global readiness or presentation. A truthful Rust binding
would need a fixed-tip Swift bridge and explicit host configuration, which violates this audit's
no-Swift boundary. B191 therefore adds no wrapper or code and leaves row
`092-commerce-services-tipkit-only-if-system-tipkit-behavior-is-specifically-requested-otherwise-framework-owned-tip-logic-may-be-portable`
at `X`. Xcode 26.6 build `17F113` / SDK 26.5 were inspected; the Xcode 27.x baseline caveat is
unchanged.

Primary API evidence: [Tip.status](https://developer.apple.com/documentation/tipkit/tip/status-swift.property),
[TipUIView](https://developer.apple.com/documentation/tipkit/tipuiview),
[TipUIView initializer](https://developer.apple.com/documentation/tipkit/tipuiview/init%28_%3Aarrowedge%3Aactionhandler%3A%29),
and [Tips.configure(_:)](https://developer.apple.com/documentation/tipkit/tips/configure%28_%3A%29)

No tests, builds, Tip presentation, datastore changes, app launches, Simulator or device calls, or
runtime probes were performed for B191
