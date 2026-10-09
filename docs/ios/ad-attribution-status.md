# iOS AdAttributionKit app-impression support snapshot

`ios-ad-attribution-status` reads `AppImpression.isSupported` on iOS 18.0 and later through a
compiler-derived C `swiftcall` thunk

`Ok(true)` or `Ok(false)` preserves Apple's result for whether AdAttributionKit supports app
impressions on the current device. It does not report postback support, network reachability,
ad-network registration, campaign eligibility, consent, token generation, or general attribution
readiness. The query creates no impression and sends no network request

The local compiler and SDK evidence uses Xcode 26.6 / iOS 26.5, below the repository's Xcode 27.x
baseline. No attribution token request, app launch, or runtime/device call was made

See the [B239 AdAttributionKit plan record](../../PLAN_CAPABILITIES_AD_ATTRIBUTION.md) and the
[package README](../../platform/ios/ios-ad-attribution-status/README.md)
