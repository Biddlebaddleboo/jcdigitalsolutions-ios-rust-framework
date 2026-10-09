# PLAN_CAPABILITIES_PROXIMITYREADER.md — D94: Row 108 Feasibility Gate

## Disposition

At the time of D94, keep row `108-extension-entitlement-capabilities-proximityreader` at `X`: the only narrow candidate, `PaymentCardReader.isSupported`, was exposed only as Swift, with no Rust, C, or Objective-C binding in the repository. B76 later added that exact device-model predicate through a compiler-derived C `swiftcall` thunk; row 108 is now `B` partial. Neither D94 nor B76 claims general ProximityReader, contactless-read, payment-processing, or Tap to Pay readiness support.

`PaymentCardReader.isSupported` could form a future platform-exclusive partial only if a supported Swift/Rust bridge is accepted. Its honest result would be a device-model predicate: Apple says it is true for iPhone XS or newer and does not check the OS version. It does not prove entitlement, region, merchant or PSP setup, account linkage, reader configuration, or transaction readiness.

## Audit scope

Inspect canonical rows 107–113 before selecting D94. Row 107 is CarKey, row 108 is ProximityReader, row 109 is LockedCameraCapture, row 110 is App Intents, row 111 is WidgetKit, row 112 is ActivityKit, and row 113 is extension bundle metadata helpers. No dedicated ProximityReader feasibility report existed at audit start; row 108 was the next unassigned target.

This report assesses only whether row 108 has an honest bounded API surface for Rust. It does not implement an API or revise the canonical capability manifest.

## Installed SDK and binding evidence

- The inspected toolchain is Xcode 26.6, build `17F113`, with iPhoneOS SDK 26.5.
- The installed `ProximityReader.swiftinterface` was built with Swift 6.3.2. It declares `PaymentCardReader.isSupported` as `public static let isSupported: Swift.Bool`; `PaymentCardReader` is available from iOS 15.4, so that is the getter's API floor.
- The framework has a Swift module interface, module map, and `.tbd`. Its public `ProximityReader.h` exports only framework version symbols; it declares no `PaymentCardReader` or ProximityReader operation in Objective-C.
- Local `objc2` 0.6.5 generated-framework coverage marks `ProximityReader` as `Swift-only`. Searches of the repository `bindings/`, `crates/`, `platform/`, and local Cargo registry found no generated Rust binding or existing ProximityReader adapter.
- Other relevant reader operations do not make a smaller Rust-callable seam: `PaymentCardReader.prepare(using:)` is async and throwing, available from iOS 16.0, takes a PSP-signed token, and returns a live reader session. `isAccountLinked(using:)` is async and throwing, available from iOS 16.4, and also needs a reader instance and token.

## Meaning and limits of the narrow candidate

Apple documents `PaymentCardReader.isSupported` as a Boolean about whether the device model supports Tap to Pay on iPhone. A true value requires iPhone XS or newer; the property does not check the OS version. Name any future Rust value to preserve this meaning, such as `tap_to_pay_device_model_supported`; do not name it `tap_to_pay_available`, `reader_ready`, or `payment_supported`.

The predicate does not report current OS compatibility, supported region, entitlement provisioning, participating payment service provider, provider certification/configuration, merchant acceptance of terms, account linkage, device preparation, or a successful card read. In particular, it is not a hardware check for any generic NFC or contactless-payment operation.

Apple documents `prepare(using:)` as configuration for payment or loyalty-card reads. It accepts a valid token from a participating payment service provider, may update device configuration over a network, may take up to two minutes on initial configuration, throws on failure, and yields a session needed for reads. Those async, service-owned, session and event semantics are outside a scalar status slice.

## Entitlement and host-app boundary

- To integrate and use Tap to Pay on iPhone, Apple requires the `com.apple.developer.proximity-reader.payment.acceptance` entitlement. The app must request it through an organization-level Apple Developer account; Apple reviews the request and provisions an approved capability. TestFlight and App Store distribution require a separate distribution entitlement.
- Apple requires coordination with a participating Level 3-certified payment service provider. Region and provider rules may also set an OS floor beyond the framework symbol's iOS 15.4 availability.
- The merchant must accept Tap to Pay terms through an API that presents a system sheet. The app then obtains and securely uses a PSP token to prepare the reader. Payment collection presents system UI to the customer, and the app sends encrypted read data to its provider for transaction processing.
- The entitlement documentation describes the requirement for Tap to Pay integration and use; it does not state that reading `PaymentCardReader.isSupported` itself requires that entitlement. This report makes no such claim.
- No separate usage-description key is established by the cited Tap to Pay integration and entitlement docs. No plist key or additional entitlement is inferred here.

These host, distribution, provider, merchant-consent, customer-presentment, and payment-service obligations mean a model predicate cannot represent the overall capability as ready for use.

## Initial feasibility result

At the time of D94, no Rust-callable ProximityReader slice existed, so the report recommended keeping row 108 at `X`. D94 did not validate a Swift ABI bridge; it scoped only the exact iOS 15.4+ device-model predicate and rejected OS, region, entitlement, account, reader, or payment-readiness claims.

Before any broader implementation, a separate scope must establish the approved Swift/Rust bridge, eligible merchant product, Apple-granted entitlement and distribution profile, supported region and PSP, token ownership, async error and cancellation mapping, session lifetime, user-present UI, and device validation. Do not use an undocumented Swift symbol, synthesize a fake C API, or substitute PassKit, CoreNFC, Apple Pay, or SecureElementCredential.

## Evidence and checks

- Read-only toolchain checks: `xcode-select -p`, `xcodebuild -version`, and `xcrun --sdk iphoneos --show-sdk-path` returned the Xcode 26.6 installation, build `17F113`, and iPhoneOS 26.5 SDK.
- Inspected `System/Library/Frameworks/ProximityReader.framework/Modules/ProximityReader.swiftmodule/arm64e-apple-ios.swiftinterface`, `Headers/ProximityReader.h`, `Modules/module.modulemap`, and `ProximityReader.tbd` in that SDK.
- Inspected the local `objc2-0.6.5/src/topics/about_generated/list_unsupported.md` entry and searched `bindings/`, `crates/`, `platform/`, and the local Cargo registry for ProximityReader declarations.
- Inspected capability rows 107–113 in `docs/capabilities/capability-status.json`; no status, metadata, or count was changed.
- No tests, builds, link probes, entitlements, runtime calls, reader configuration, payment reads, or user prompts ran.

## B76 follow-up

B76 later implemented only `PaymentCardReader.isSupported` through a compiler-derived C `swiftcall` thunk in `platform/ios/ios-proximity-reader`. Device and arm64 Simulator compiler oracles, target checks/strict Clippy, rustdoc, and release link/import/deployment checks passed on Xcode 26.6 / SDK 26.5. The package preserves the D94 boundary: the result is only the iPhone model predicate, not OS-version support or Tap to Pay readiness. Link probes were inspected, not executed; no live device query, entitlement, merchant, region, PSP, or payment flow was exercised. Row 108 is now `B` partial; see [B76](PLAN_IOS_PROXIMITYREADER.md) and [G92](PLAN_VALIDATION_IOS_PROXIMITYREADER.md)

## Apple and binding references

- [ProximityReader framework](https://developer.apple.com/documentation/proximityreader)
- [`PaymentCardReader.isSupported`](https://developer.apple.com/documentation/proximityreader/paymentcardreader/issupported)
- [Adding support for Tap to Pay on iPhone](https://developer.apple.com/documentation/proximityreader/adding-support-for-tap-to-pay-on-iphone-to-your-app)
- [Setting up the Tap to Pay on iPhone entitlement](https://developer.apple.com/documentation/proximityreader/setting-up-the-entitlement-for-tap-to-pay-on-iphone)
- [`PaymentCardReader.prepare(using:)`](https://developer.apple.com/documentation/proximityreader/paymentcardreader/prepare%28using%3A%29)
- [`PaymentCardReader.Token`](https://developer.apple.com/documentation/proximityreader/paymentcardreader/token)
- [objc2 generated framework catalog](https://github.com/madsmtm/objc2/blob/main/crates/objc2/src/topics/about_generated/list_unsupported.md)
