# iOS AdAttributionKit app-impression support snapshot

This crate exposes only `AppImpression.isSupported` through a compiler-derived C `swiftcall`
thunk

Apple defines the value as whether AdAttributionKit supports app impressions on the current
device. It does not report postback support, attribution readiness, ad-network registration,
campaign eligibility, consent, token generation, network access, or a successful future operation.
The query creates no impression and sends no network request

The API floor is iOS 18.0; the `AppImpression` type and framework begin at iOS 17.4. Older iOS and
non-iOS targets return `AdAttributionError::NativeApiUnavailable`. The crate adds no Swift source,
Objective-C selector, or external Rust dependency

See [the focused guide](../../../docs/ios/ad-attribution-status.md) and
[B239 plan record](../../../PLAN_CAPABILITIES_AD_ATTRIBUTION.md)
