# PLAN_CAPABILITIES_AD_ATTRIBUTION.md — Workstream D74: Row 089 Feasibility Gate

## Result

Keep capability row `089-commerce-services-adattributionkit-adservices-as-applicable` at `X`. Apple exposes a small, nonprompting support query, `AppImpression.isSupported`, with an iOS 18.0 API floor. Its sole public declaration is Swift; the installed SDK has no C or Objective-C declaration, and the generated Rust bindings have no AdAttributionKit crate. This project bars Swift source and direct Swift ABI calls, so no Rust-callable slice exists under the current rules.

`Postback.isSupported` is another Swift-only Boolean at iOS 18.0, but it reports support for postbacks only. Neither property reports ad-network registration, attribution service reachability, campaign eligibility, postback delivery, or general AdAttributionKit readiness.

AdServices has an Objective-C token-generation method at iOS 14.3 and an online `objc2-ad-services` 0.3.2 binding. It is not a status query: it attempts to create an attribution token, requires unimpeded internet access, and the SDK header warns against token generation on Simulator. Keep this operation out of a low-data support slice.

## Objective

Determine whether row 089 has a public, nonprompting, low-data status or support query that Rust can call without Swift, a user-data operation, an entitlement, or an attribution service lifecycle.

## SDK and binding evidence

Inspected Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5.

- `AdAttributionKit.framework/Modules/AdAttributionKit.swiftmodule/arm64e-apple-ios.swiftinterface` declares `AppImpression` at iOS 17.4 and `AppImpression.isSupported` in an iOS 18.0 extension. The property is a static `Swift.Bool` getter.
- The same Swift interface declares `Postback` at iOS 17.4 and `Postback.isSupported` in an iOS 18.0 extension. That property is also a static `Swift.Bool` getter.
- `AdAttributionKit.framework/Headers/AdAttributionKit.h` is an empty umbrella header. The module interface exposes Swift declarations only; no C or Objective-C entry point for either support property appears in the installed SDK.
- `AdServices.framework/Headers/AAAttribution.h` declares `AAAttribution` and `+attributionTokenWithError:` at iOS 14.3. The method may return `AAAttributionErrorCodePlatformNotSupported`, but only as an error from the token-generation operation, not as a separate query.
- The same header documents `AAAttributionErrorCodeNetworkError` when internet access is unavailable and says token generation needs unimpeded internet access. Its comment says not to use Simulator when generating a token.
- `objc2-ad-services` 0.3.2 exists as a generated Rust binding with the `AAAttribution` feature. No `objc2-ad-attribution-kit` crate or generated binding for `AppImpression.isSupported` was found. The AdServices crate is not in this workspace or local Cargo source cache.

The Swift interface gives a framework API floor, not a deployment-target or SDK-link baseline for this repository. The only floor verified here is iOS 18.0 for `AppImpression.isSupported` and `Postback.isSupported`; the enclosing `AppImpression` and `Postback` declarations begin at iOS 17.4.

## Privacy, consent, and host scope

Apple states that apps do not need App Tracking Transparency before they call AdAttributionKit ad-attribution APIs, and may call those APIs regardless of tracking authorization status. This does not grant consent for cross-app tracking: apps that perform tracking must still follow the App Tracking Transparency rules.

`AppImpression.isSupported` is a scalar framework-support check. Its API contract does not create an impression, return an attribution token, register an ad network, or read campaign data. No usage-description key or entitlement for this static query appears in the inspected public docs or SDK declaration. This is not a claim that every AdAttributionKit operation has no host configuration.

Actual attribution use has broader service and app obligations:

- Ad networks register with Apple and sign impression JWS values.
- Publisher apps configure accepted ad-network identifiers and display signed ads.
- View attribution uses `beginView()` / `endView()` with visibility requirements; tap attribution uses a `UIEventAttributionView` and an `AppImpression` tap method.
- Advertised apps update conversion values; optional postback copies require a server endpoint.
- AdServices token use creates a token that an app or measurement provider may send to Apple's attribution endpoint. The token has a 24-hour TTL, and the resulting payload detail depends on privacy settings, including the device-level “Allow Apps to Request to Track” setting.

A `true` support value does not prove any of those registrations, configurations, event lifecycles, network paths, user choices, campaign matches, or postback results.

## Feasibility and next evidence

There is no honest direct Rust implementation of the static support check under the current no-Swift rule. Do not call mangled Swift symbols, add an unreviewed Swift shim, use the AdServices token method as a probe, or map token-generation errors to a general support status.

If the project later approves a minimal Swift bridge with a stable reviewed C ABI, a future workstream may assess a Boolean-only `AppImpression.isSupported` adapter. That work must retain the iOS 18.0 API floor, use the exact meaning “supports app impressions on this device,” and keep postback support distinct. It must not claim that a full ad-attribution backend exists.

## Deferred work

- No portable capability contract, iOS crate, Swift shim, Objective-C wrapper, C ABI, AdServices token request, impression creation, view/tap handling, conversion update, postback, ad-network setup, or live service probe.
- No Cargo manifest or lockfile, canonical capability manifest, CI, aggregate plan, shared index, or matrix edit.
- No tests or runtime probes.

## Apple and binding references

- [AdAttributionKit overview](https://developer.apple.com/documentation/AdAttributionKit)
- [AppImpression](https://developer.apple.com/documentation/adattributionkit/appimpression)
- [AppImpression.isSupported](https://developer.apple.com/documentation/adattributionkit/appimpression/issupported)
- [Postback.isSupported](https://developer.apple.com/documentation/adattributionkit/postback/issupported)
- [Generating JWS impressions](https://developer.apple.com/documentation/adattributionkit/generating-jws-impressions)
- [AdServices overview](https://developer.apple.com/documentation/AdServices)
- [AAAttribution](https://developer.apple.com/documentation/adservices/aaattribution)
- [AAAttribution.attributionToken()](https://developer.apple.com/documentation/adservices/aaattribution/attributiontoken%28%29)
- [App Tracking Transparency](https://developer.apple.com/documentation/apptrackingtransparency)
- [`objc2-ad-services` 0.3.2](https://docs.rs/objc2-ad-services/0.3.2/objc2_ad_services/)

## B185 follow-up: AdAttributionKit support contract remains Swift-only

Rechecked row 089 against the installed iOS 26.5 interface, framework headers, framework stub,
and current Apple API docs. `AppImpression.isSupported` has a precise and useful contract —
whether the framework supports app impressions on this device — and `Postback.isSupported`
reports postback support separately. Both remain Swift static properties with no public
Objective-C/C declaration in the SDK. `AdAttributionKit.framework/Headers/AdAttributionKit.h`
is only an umbrella header; the Swift interface declares these getters as `Swift.Bool` and does
not mark them `@objc`. The framework stub exposes Swift-mangled symbols, not a C ABI suitable for
the project’s Rust-only boundary. No generated `objc2-ad-attribution-kit` binding is available.

The Objective-C alternative, `AAAttribution.attributionTokenWithError:`, does not fill this gap:
the public `AAAttribution.h` declares token generation only, and Apple's documentation says it
generates a 24-hour attribution token for use in the attribution flow. Its `NetworkError` means
the server cannot provide a token without unimpeded internet access. Treating this service/token
operation as a support query would both change the data/network contract and conflate platform
support with token generation success.

No Rust-callable narrow surface is justified under the current no-Swift rule. B185 makes no code,
binding, dependency, manifest, or row-status change; row
`089-commerce-services-adattributionkit-adservices-as-applicable` remains `X`. This audit used
Xcode 26.6 build `17F113` and iPhoneOS SDK 26.5; the Xcode 27.x baseline caveat is unchanged.

Primary API evidence: [AppImpression.isSupported](https://developer.apple.com/documentation/adattributionkit/appimpression/issupported),
[Postback.isSupported](https://developer.apple.com/documentation/adattributionkit/postback/issupported),
[AAAttribution (Objective-C)](https://developer.apple.com/documentation/adservices/aaattribution?language=objc),
[AAAttribution.attributionToken()](https://developer.apple.com/documentation/adservices/aaattribution/attributiontoken%28%29),
and [AAAttribution NetworkError](https://developer.apple.com/documentation/adservices/aaattributionerror/networkerror)

No tests, builds, token requests, network calls, app launches, or runtime/device probes were
performed for B185
