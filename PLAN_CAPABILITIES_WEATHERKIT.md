# PLAN_CAPABILITIES_WEATHERKIT.md — D79: WeatherKit audit for row 091

## Scope

Audit row `091-commerce-services-weatherkit-through-rest-native-http-where-appropriate` for a public Rust or Objective-C binding path, host requirements, and a credible bounded implementation

This audit does not change the canonical capability matrix, global docs, CI, Cargo manifests, or source code

## Status and recommendation

Keep row 091 X for a framework-specific Rust WeatherKit package at this time. The native WeatherKit API is a Swift-only surface, and this repository has no Rust binding for it. Apple also offers a documented REST service that a Rust HTTP client can call, so this is not a claim that WeatherKit data is impossible to access from Rust

The existing `framework-network` package provides generic foreground HTTP transport, not WeatherKit authentication, JSON decoding, attribution, or typed weather values. A request-builder-only package or a raw response wrapper would not establish a useful WeatherKit value contract. A full REST client is technically feasible, but it needs an explicit host-owned developer-token boundary, data-model/decode scope, and attribution contract. No such package boundary is selected here, so do not classify this row as implemented

Do not route Rust through a hand-written Objective-C shim for `WeatherService`: the installed public SDK has no Objective-C header for WeatherKit, and its native service methods use Swift async/throws and generic Swift values. Do not use private Swift ABI symbols

## Native API and SDK evidence

Inspection used Xcode 26.6 build 17F113 and iPhoneOS 26.5 SDK

- SDK path: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/WeatherKit.framework`
- The framework contains `Modules/WeatherKit.swiftmodule/arm64e-apple-ios.swiftinterface`, its `.swiftdoc`, and `WeatherKit.tbd`. It has no public `Headers` directory or Objective-C module map
- The public Swift interface declares `WeatherService` as a Swift class, available on iOS 16.0. It exposes `shared`, an initializer, async-throwing `attribution`, and async-throwing `weather(for: CLLocation)` plus generic `WeatherQuery` overloads. The interface also marks later statistics APIs as iOS 18.0
- The `.tbd` exports Swift-mangled symbols such as `_$s10WeatherKit...`, not a public Objective-C selector/function surface suitable for `objc2`
- The cached `objc2` 0.6.5 generated-framework catalog at `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_unsupported.md` lists `WeatherKit` as Swift-only. The local Cargo registry has no WeatherKit binding crate
- Native WeatherKit's documented minimum OS version is iOS 16.0. This is an API floor only; no Rust link or runtime probe was run

`WeatherService` accepts a caller-provided `CLLocation`; it does not itself represent a location-permission query. If an app obtains that location from Core Location, location consent and usage-description requirements belong to that separate Core Location feature

## Native host setup, service limits, and attribution

- Apple's `WeatherKit Entitlement` documentation identifies `com.apple.developer.weatherkit` as a Boolean entitlement whose default is `NO`; enable the WeatherKit capability in Xcode for an app that uses the native framework
- WeatherKit access is tied to Apple Developer Program membership. The Apple getting-started page lists up to 500,000 API calls per month per membership before an optional paid subscription for more calls
- Native `WeatherService.attribution` is an async-throwing API. A client that displays Apple weather data must honor Apple's attribution rules; the app remains responsible for the display and legal attribution experience
- Dataset availability varies by location. Minute precipitation and alerts are not available everywhere; the caller must treat service errors and unavailable datasets as ordinary outcomes
- This slice makes no promise of network availability, service response, forecast coverage, or quota beyond Apple's published service terms

## REST route and Rust boundary

Apple documents two relevant REST routes:

- `GET /api/v1/availability/{latitude}/{longitude}` to query datasets available at a coordinate
- `GET /api/v1/weather/{language}/{latitude}/{longitude}` to request weather data for a coordinate

The REST API has no iOS deployment floor in the cited endpoint documentation; a host must supply HTTPS transport and a valid caller-selected coordinate/language. This does not lower the native Swift API's iOS 16.0 floor

Every REST request requires an `Authorization: Bearer` developer token. Apple's authentication instructions require an ES256-signed JWT with the documented key identifier and issuer, subject, issue-time, and expiry claims. The setup uses a WeatherKit key and Service ID. Apple explicitly says never to distribute the private key and advises apps or websites to use an authenticated service to create and sign tokens. Do not embed the WeatherKit signing key in an iOS app or generate the developer signature in the client

Apple states that WeatherKit REST use requires attribution. A future Rust REST package must make the host's token source and attribution responsibilities explicit, handle ordinary HTTP/service errors, validate caller input, and define whether it returns decoded typed values or only raw response bytes. `framework-network` already has `HttpClient`, `HttpRequest`, and `HttpResponse`; its `HttpBackend` is caller supplied, and its response body is opaque bytes. It does not supply JWT signing, JSON decoding, WeatherKit models, retry, cache, or attribution UI

The REST documentation targets web apps and other platforms and says native Apple apps should use the native WeatherKit framework. Using REST inside an iOS app therefore needs an explicit product/host decision even though the REST service is technically accessible through HTTP

## Acceptance boundary

This workstream establishes only:

- the native WeatherKit API is Swift-only in the inspected SDK and unavailable through a generated public Objective-C/Rust binding
- native `WeatherService` starts at iOS 16.0 and has Swift async/throws and generic API requirements
- Apple's REST route is a viable Rust HTTP architecture when a trusted host supplies its signed developer token and the product handles attribution and response semantics
- generic HTTP support alone is not a WeatherKit implementation
- row 091 remains X until a concrete REST client boundary is selected and implemented, or a public Rust-accessible native API becomes available

This workstream does not claim a WeatherKit fetch, typed forecast value, authorization check, user-location access, map/weather UI, successful response, or native SDK parity

## Apple and local sources

- [WeatherKit](https://developer.apple.com/documentation/weatherkit)
- [WeatherService](https://developer.apple.com/documentation/weatherkit/weatherservice)
- [Get Started with WeatherKit](https://developer.apple.com/weatherkit/)
- [WeatherKit REST API](https://developer.apple.com/documentation/weatherkitrestapi)
- [Request authentication for WeatherKit REST API](https://developer.apple.com/documentation/weatherkitrestapi/request-authentication-for-weatherkit-rest-api)
- [WeatherKit Entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.weatherkit)
- Local SDK interface: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/WeatherKit.framework/Modules/WeatherKit.swiftmodule/arm64e-apple-ios.swiftinterface`
- Local SDK exports: `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk/System/Library/Frameworks/WeatherKit.framework/WeatherKit.tbd`
- Local generated-binding support catalog: `$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-0.6.5/src/topics/about_generated/list_unsupported.md`
- Existing portable HTTP contract: `crates/framework-network/src/lib.rs`

## Audit record

The audit inspected the SDK file list, the `WeatherService` declarations and availability annotations in the Swift interface, the `.tbd` export form, the local `objc2` unsupported-framework catalog, the existing `framework-network` contract, and Apple's primary WeatherKit, REST authentication, and entitlement documentation

No source edit, package edit, test, build, link probe, or runtime probe was made. `xcrun --sdk iphoneos --show-sdk-version` reported `26.5`; `xcodebuild -version` reported `Xcode 26.6` and build `17F113`
