# PLAN_CAPABILITIES_MARKETPLACEKIT.md — D90: MarketplaceKit row 104 audit

## Scope

Report-only audit of row `104-extension-entitlement-capabilities-marketplacekit`. Inspect the installed iOS SDK, local/generated Rust and C/Objective-C bindings, Swift interfaces, and Apple primary documentation. Do not edit the canonical matrix, shared indexes, CI, Cargo files, source, or this report's neighboring plans. No tests or builds are in scope

## Recommendation

Keep row 104 at `X` for the current Rust implementation. MarketplaceKit is a substantial, region-gated app-distribution service, not a general device feature bit. Its main app-management APIs have entitlement and host-role limits, and its user-facing installation paths require user action and system UI

There is a documented source query that appears no-prompt: `AppDistributor.current` reports whether the app was installed from the App Store, TestFlight, an alternative marketplace, the developer's website, or another source. This is useful for source-dependent app behavior; it does not report marketplace entitlement, Apple approval, device or account eligibility, regional marketplace availability, or ability to install or vend apps. The public API is Swift-only `async throws`; the installed SDK has no public C/Objective-C declaration or generated Rust binding for it. The current Swift ABI work also does not provide a supported Rust async call path. Thus the query does not yield a supported Rust slice that moves this row from `X`

`AppDistributor.eligibilityRegion` and `AppLibrary.catalogRegion` are additional asynchronous region values, not marketplace-readiness predicates. Their meanings are specific to transaction reporting and catalog selection, respectively. Neither establishes entitlement or permission state

## Installed SDK and binding evidence

The local toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5

- Framework: `iPhoneOS.sdk/System/Library/Frameworks/MarketplaceKit.framework`
- The public umbrella header `Headers/MarketplaceKit.h` has only its framework copyright comment; it declares no C functions, Objective-C classes, protocols, or selectors. The module map exports that header, while the public API is in `Modules/MarketplaceKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- The installed `MarketplaceKit.tbd` exports Swift-mangled symbols; those exports do not provide a documented C ABI for these APIs
- The Swift interface declares `AppDistributor.current` as a static `async throws` property on the iOS 17.4 API surface. The enum cases identify `.appStore`, `.testFlight`, `.marketplace(String)`, `.web` (iOS 17.5+), and `.other`
- The interface declares `AppDistributor.eligibilityRegion` as an async optional string introduced in iOS 26.4. Apple describes it as the device's current region code and documents its use for Core Technology Commission transaction-reporting eligibility
- `AppLibrary` is a Swift `@MainActor` class introduced in iOS 17.4. Its `current` accessor and iOS 26.2 `catalogRegion` getter are declared `nonisolated`; `catalogRegion` is still async. Apple documents it as the device's current region code, used by an alternative marketplace to select its catalog, and says it is `nil` when the app is not an alternative marketplace
- `AppLibrary.searchTerritory` is a separately settable marketplace search filter, not a device-region or support query. Setting it changes which apps system-wide search returns
- `MarketplaceAppExtension` is a Swift protocol introduced in iOS 26.0. The earlier Swift `MarketplaceExtension` and `MarketplaceExtensionConfiguration` protocols were introduced in iOS 17.4 and are deprecated in iOS 26.0
- `_MarketplaceKit_UIKit.framework` contains a Swift `ActionButton` class marked `@objc`, introduced in iOS 17.4 and `@MainActor`. Its action enum and install configuration are Swift values; this is a user-interaction control, not a status-query bridge
- No `objc2-marketplace-kit` generated crate is present in the local Cargo registry source cache. Repository Rust, C, and Objective-C source has no MarketplaceKit binding or adapter
- The existing Swift ABI evidence is insufficient for `AppDistributor.current`: C4 records that Swift-generated C++ headers omit public async entries, and C6 found no documented public task-entry/context/resume contract or Rust `Future` adapter. C1's retained-class proof does not provide direct Swift method calls or async calls; see `PLAN_SWIFT_ABI.md`, `PLAN_SWIFT_ABI_ASYNC.md`, and `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`

## API roles and status-query limits

### Installation-source value

Apple documents `AppDistributor.current` for apps installed from multiple sources so they can adjust their own behavior at launch. The query returns an installation-source enum, not a marketplace support result. In Xcode development builds, the `MARKETPLACES` build setting can override the value for source-path simulation; a development result can therefore be simulated rather than evidence of production distribution

The API shape and documented use have no presentation or permission request, so this appears to be a no-prompt query. That is a source-based inference from the public property and its documentation; it is not a guarantee about marketplace install workflows

### Region values

- `AppDistributor.eligibilityRegion` reports the device's region code; Apple documents it for transaction-reporting eligibility. It does not report Apple approval, an entitlement profile, whether a marketplace may operate in that region, or whether a particular installation will succeed
- `AppLibrary.catalogRegion` reports a device region code for a marketplace's catalog choice and is `nil` when the app is not an alternative marketplace. The `AppLibrary` class is `@MainActor`, and iOS ignores calls to it for apps without `com.apple.developer.marketplace.app-installation` or `com.apple.developer.browser.app-installation`
- `AppLibrary.searchTerritory` is an optional two-letter region filter selected by the marketplace; it is not the device's actual region or marketplace eligibility
- None of these values is a generic entitlement check, user authorization status, or proof of a working marketplace server

## User-facing operations and host boundary

- The principal marketplace use is app discovery and distribution: receive and retrieve notarized apps, deliver licenses and packages from a server, install/update apps, support restore, and integrate with system app-management features
- `AppLibrary` install/update methods require an authorized marketplace or browser host. Apple says iOS ignores `AppLibrary` calls when the app lacks `com.apple.developer.marketplace.app-installation` or `com.apple.developer.browser.app-installation`
- Marketplace app installation is user-facing. Apple requires an install request that presents the system sheet to originate from the user's interaction with MarketplaceKit's `ActionButton`; the user confirms the install in system UI
- Browser installation from a web page is a separate path that uses `requestAppInstallationFromBrowser(for:referrer:)`, the `com.apple.developer.browser.app-installation` entitlement, and a user-initiated webpage action before MarketplaceKit presents the install sheet
- A marketplace host also needs a new app identity, website and server components, developer relationships, license and package endpoints, and relevant extension callbacks. In iOS 26+, `MarketplaceAppExtension` supplies Swift async callbacks for server headers, app-version lookup, and automatic updates; earlier `MarketplaceExtension` is deprecated
- Alternative-distribution apps also declare the `MKSellsDigitalGoods` target property and, where applicable, report transactions with `TransactionReporting`; these are distribution/compliance duties, not support or entitlement queries. Apps installed from alternative marketplaces may need different commerce and service APIs than App Store installs
- The MarketplaceKit error enum includes installation restrictions and marketplace-denial errors, but these describe attempted operations. They are not a prompt-free query for entitlement, approval, or eligibility

## Region, entitlement, and distribution gates

- `com.apple.developer.marketplace.app-installation` is a managed entitlement for an app that vends other apps as an alternative marketplace. Apple approval and a distribution entitlement/profile are separate from SDK availability
- `com.apple.developer.browser.app-installation` enables a browser to install alternative-distribution apps from a website; it is not the marketplace-vending entitlement
- In the European Union, Apple requires authorization to operate a marketplace. The marketplace entitlement path is available to EU users on iOS 17.4+ and iPadOS 18+; website distribution for non-marketplace apps has a separate iOS 17.5+ floor
- In Japan, marketplace operation and distribution are available beginning with iOS 26.2, subject to region-specific entitlement approval and Apple Developer Program terms
- In Brazil, marketplace operation and distribution are available beginning with iOS 26.5, also subject to separate regional approval and terms
- Approval in one region does not confer approval in another. Apple requires a separate regional entitlement request; installation, notarization, server, App Store Connect, and business obligations remain outside a local Rust capability snapshot
- MarketplaceKit's framework/API availability does not imply that an app, user, device, account, storefront, region, or developer is approved or eligible to operate or use a marketplace

## Disposition

Row 104 remains `X` for this Rust workstream. `AppDistributor.current` is a well-defined Swift installation-source status candidate with no documented prompt, but its async Swift-only interface has no generated Rust or C/Objective-C path, and the current supported Swift ABI work blocks a Rust async wrapper. The region properties are narrower compliance/catalog inputs, while installation and marketplace services are entitlements-, region-, server-, and user-interaction-gated. Revisit only if a documented public Rust-callable async route or a supported Rust-reachable synchronous/C/Objective-C status API appears; do not infer general MarketplaceKit support from an OS version, region code, or installation-source value

## Audit checks

- Read the installed iPhoneOS SDK 26.5 module map, `MarketplaceKit.h`, Swift interfaces, and framework `.tbd`; record Xcode 26.6 build `17F113`
- Search the local Cargo registry and repository Rust/C/Objective-C source for generated bindings or adapters; none were found
- Review Apple MarketplaceKit, AppLibrary, AppDistributor, installation, browser-installation, entitlement, and region-specific distribution documentation
- No tests, builds, link probes, app installs, entitlements, or runtime queries were run

## Root integration notes

No package, dependency, workspace, lock, CI, docs-index, or matrix change is required. Keep row 104 at `X`; if root revises the row's reason, use the D90 blocker rather than recording iOS 17.4 as an implemented support floor. No capability counts change

## Apple primary sources

- [MarketplaceKit overview](https://developer.apple.com/documentation/marketplacekit)
- [Distributing your app on an alternative app marketplace](https://developer.apple.com/documentation/marketplacekit/distributing-your-app-on-an-alternative-marketplace)
- [Distributing your app from your website](https://developer.apple.com/documentation/marketplacekit/distributing-your-app-from-your-website)
- [`AppDistributor`](https://developer.apple.com/documentation/marketplacekit/appdistributor)
- [`AppDistributor.eligibilityRegion`](https://developer.apple.com/documentation/marketplacekit/appdistributor/eligibilityregion)
- [`AppLibrary`](https://developer.apple.com/documentation/marketplacekit/applibrary)
- [`AppLibrary.catalogRegion`](https://developer.apple.com/documentation/marketplacekit/applibrary/catalogregion)
- [`AppLibrary.searchTerritory`](https://developer.apple.com/documentation/marketplacekit/applibrary/searchterritory)
- [Installing apps from an alternative marketplace](https://developer.apple.com/documentation/marketplacekit/installing-apps-from-an-alternative-marketplace)
- [Enabling alternative distribution app installation in a browser](https://developer.apple.com/documentation/marketplacekit/enabling-alternative-distribution-app-installation-in-a-browser)
- [Creating an alternative app marketplace](https://developer.apple.com/documentation/marketplacekit/creating-an-alternative-app-marketplace)
- [Participating in alternative distribution for specific regions](https://developer.apple.com/documentation/marketplacekit/participating-in-alternative-distribution-for-specific-regions)
- [`com.apple.developer.marketplace.app-installation`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.marketplace.app-installation)
- [`com.apple.developer.browser.app-installation`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.browser.app-installation)
- [Operating an alternative app marketplace in the EU](https://developer.apple.com/support/alternative-app-marketplace-in-the-eu/)
- [Changes to iOS in Japan](https://developer.apple.com/support/app-distribution-in-japan)
- [Changes to iOS in Brazil](https://developer.apple.com/support/app-distribution-in-brazil)
- [Apple Developer Program License Agreement](https://developer.apple.com/support/terms/apple-developer-program-license-agreement/)

## Internal evidence

- Installed SDK root: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk`
- `MarketplaceKit.framework/Headers/MarketplaceKit.h` and `Modules/MarketplaceKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- `_MarketplaceKit_UIKit.framework/Modules/_MarketplaceKit_UIKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- `MarketplaceKit.framework/MarketplaceKit.tbd`
- Swift ABI limits: `PLAN_SWIFT_ABI.md`, `PLAN_SWIFT_ABI_ASYNC.md`, `PLAN_SWIFT_ABI_ASYNC_RUNTIME.md`
- Canonical row reviewed only: `docs/capabilities/capability-status.json`, id `104-extension-entitlement-capabilities-marketplacekit`

## B230 follow-up: no C/Objective-C marketplace readiness query

Rechecked the installed iOS 26.5 MarketplaceKit headers and module interfaces for a native Rust-callable operation. `MarketplaceKit.framework/Headers/MarketplaceKit.h` remains declaration-free beyond its framework comment. The framework interface exposes `AppDistributor.current` as `async throws`; `eligibilityRegion` (iOS 26.4) and `AppLibrary.catalogRegion` (iOS 26.2) are also asynchronous Swift properties. They report installation source or narrowly scoped region values, not app entitlement, Apple approval, device/account eligibility, or installation readiness.

The apparent Objective-C exception is `_MarketplaceKit_UIKit.ActionButton`, a `@MainActor @objc` UIKit control. Apple defines it as the user-interface element through which a person installs, updates, or launches apps; iOS validates that an install request came from interaction with this control. Its inherited `UIControl.isEnabled` is control interaction state, not MarketplaceKit support, entitlement, or marketplace authorization. Wrapping it would require a real user-facing install flow and would not provide a status operation. The framework's extension and catalog APIs remain Swift protocol/async/value surfaces; no Objective-C/C query or generated Rust binding is available.

Decision: no implementation or dependency change for B230; keep row `104-extension-entitlement-capabilities-marketplacekit` at `X`. The existing async installation-source and region values do not cross the supported Rust ABI boundary, and mapping them to generic MarketplaceKit availability would be false. A future slice requires an explicitly supported Rust/Swift async bridge or a new public C/Objective-C query with documented semantics. Evidence remains Xcode 26.6 build `17F113` / iOS SDK 26.5, below the repository's Xcode 27.x baseline.

Evidence: installed `MarketplaceKit.framework/Headers/MarketplaceKit.h`, `MarketplaceKit.swiftmodule/arm64e-apple-ios.swiftinterface`, and `_MarketplaceKit_UIKit.swiftmodule/arm64e-apple-ios.swiftinterface`; [Apple `AppDistributor.current` usage](https://developer.apple.com/documentation/marketplacekit/distributing-your-app-on-an-alternative-app-marketplace), [`eligibilityRegion`](https://developer.apple.com/documentation/marketplacekit/appdistributor/eligibilityregion), [`ActionButton`](https://developer.apple.com/documentation/marketplacekit/actionbutton?changes=__5_7&language=objc), and [MarketplaceKit overview](https://developer.apple.com/documentation/marketplacekit).

No source, dependency, build, link probe, test, app install, marketplace action, UI presentation, app launch, Simulator run, or device query was performed for B230
