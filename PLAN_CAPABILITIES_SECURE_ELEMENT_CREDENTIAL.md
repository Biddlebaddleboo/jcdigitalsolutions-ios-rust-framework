# PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md — D92: Row 106 Feasibility Gate

## Status

D92 found one narrow status-like API: `CredentialSession.isEligible`, an iOS 18.1+ async Swift property that reports whether the app or extension is eligible to start a credential session. It is not a general Secure Element hardware or availability query. Apple requires the restricted Secure Element Credential entitlement for calls to the framework, and a session start requires foreground app state and user approval. The framework has no generated Rust binding or Objective-C/C surface in the inspected SDK. Keep row `106-extension-entitlement-capabilities-secureelementcredential` at `X`; no implementation or matrix change is part of D92.

## Objective

Determine whether row 106 has a useful, bounded Rust-callable operation that does not require a qualified NFC & Secure Element Platform app, privileged entitlement, credential session, applet, or user-present transaction lifecycle.

## Installed SDK and binding evidence

- Inspected Xcode 26.6 build `17F113` and the iPhoneOS 26.5 SDK. The installed SecureElementCredential Swift interface was built with Swift 6.3.2.
- `SecureElementCredential.framework` has a Swift module interface and `.tbd`, but no public Objective-C headers or module map in the installed SDK. `CredentialSession` is a Swift `actor`; `CredentialSession.isEligible` is `static var isEligible: Bool { get async throws }`, available from iOS 18.1.
- Apple defines `isEligible` as whether the app or app extension is eligible to start a credential session, and recommends it before `startSession()`. It is not a generic Secure Element, NFC, hardware, credential-presence, or app-permission status. The property may throw; the SDK interface does not flatten it to a synchronous Boolean query.
- `CredentialSession.startSession() async throws` is also available from iOS 18.1. The actor exposes session state and an `AsyncStream` of events; credential list, provision, delete, wired mode, card emulation, and presentment assertion operations all depend on a live session. `secureElementInfo` is available from iOS 18.4 and also requires a session.
- The installed `objc2` 0.6.5 generated-framework catalog at `src/topics/about_generated/list_unsupported.md` classifies `SecureElementCredential` as Swift-only. No generated Rust binding or repository Rust/C/Objective-C declaration for this framework was found in the local registry or source tree. The installed UIKit and SwiftUI additions are separate Swift module interfaces; their transaction paths use SwiftUI `View`, `UIKit.UIScene`, async methods, or Swift scene-delegate values.

## Entitlement, eligibility, and lifecycle boundary

- `com.apple.developer.secure-element-credential` is required for an app or extension to create credential sessions. Apple's `startSession()` documentation says calls to SecureElementCredential APIs without this entitlement raise `fatalError(_:file:line:)`; do not call `isEligible` as an unentitled entitlement-discovery mechanism.
- `com.apple.developer.secure-element-credential.default-contactless-app` is a separate entitlement for an app that seeks default contactless-app status. D92 does not claim every app needs this additional entitlement.
- Apple requires an eligible NFC & SE Platform use case, an Apple agreement, Apple Business Register (ABR) onboarding and an approved entitlement. The organization and app must meet territory, legal, security, product, and distribution criteria. The Secure Element applet bundle and product configuration must be registered and approved in ABR before the actual credential provisioning path. This is not an ordinary developer permission or generally available device API.
- iOS 18.1 is the API introduction floor, not a universal distribution floor. Apple's current platform page lists iPhone XS or later and territory-dependent minimums, including iOS 18.1, 18.2, 18.4, and 26.0 for different eligible territories and uses. Government ID has separate iOS 26.4 availability. Keep those distribution rules distinct from the symbol availability annotation.
- Apple documents `isEligible` as a preflight for the current device and user configuration. It does not claim to expose entitlement state, ABR approval, a registered applet, a credential in the Secure Element, NFC reader presence, or successful presentment. A positive result cannot promise that later session, credential, or transaction operations will succeed.
- `startSession()` requires the app to be foregrounded. On first credential-session access, Apple presents a privacy information sheet and asks the person to allow Secure Element access; a denial invalidates the session path until the person changes Settings. Only one session can be active per app, and the system invalidates it after a short delay when the app enters the background. These ownership and user-consent semantics do not form a general status-only API.
- Contactless presentment requires user intent and authentication through the system transaction UI. Presentment intent assertion is foreground-only and is restricted to an eligible NFC transaction intent. Do not expose or acquire it as a generic lock, NFC-ready state, or background lease.

## Feasibility result and next evidence

No general Rust-callable SecureElementCredential slice is established. `CredentialSession.isEligible` is a precise preflight for a specialized app that already has the Secure Element Credential entitlement and product scope; its Swift actor/async contract has no generated Rust binding, and entitlement absence makes calls to this framework unsafe by documented fatal-error behavior. It must not be reported as general Secure Element support or as a way to discover whether an app can request entitlement.

Consider a later, explicitly platform-exclusive package only after:

1. A concrete NFC & SE Platform use case and eligible territory are selected.
2. Apple grants the app the `com.apple.developer.secure-element-credential` entitlement and the organization completes the required agreement and ABR onboarding/product setup.
3. A supported Rust/Swift interop design covers the Swift actor, async throwing property, session lifetime, cancellation, and error mapping. Do not guess Swift ABI symbols or replace them with an unentitled raw Secure Element path.
4. The package has a scope narrower than the entire SecureElementCredential framework and preserves user consent, foreground restrictions, session invalidation, and credential ownership.
5. Device validation uses an eligible signed app, registered credential/app applet configuration, and real NFC hardware for transaction behavior. A build or symbol check cannot prove ABR provisioning, user authorization, or transaction success.

## Separate capabilities

- `SecureElementCredential` manages Apple-approved credential applets and Secure Element transactions. Its eligibility value is not equivalent to general CoreNFC tag-reader support.
- CoreNFC tag access, HCE `CardSession`, PassKit/Apple Pay, and the default contactless app path have separate APIs, entitlements, use cases, and user flows. None is a drop-in replacement for SecureElementCredential applet management; do not infer row 106 support from an adjacent NFC or payment API.

## Deferred work

- No portable `framework-payments` contract, iOS backend, Rust binding, Swift bridge, entitlement request, ABR applet or product configuration, contactless-app declaration, plist change, credential session, or transaction.
- No canonical capability manifest, Cargo/workspace/lockfile, CI, aggregate plan, global docs index, or row-count edit.
- No tests, builds, link probes, credential reads, user prompt, NFC scan, presentment assertion, or runtime probe.

## Apple and binding references

- [SecureElementCredential framework](https://developer.apple.com/documentation/secureelementcredential)
- [CredentialSession](https://developer.apple.com/documentation/secureelementcredential/credentialsession)
- [CredentialSession.isEligible](https://developer.apple.com/documentation/secureelementcredential/credentialsession/iseligible)
- [CredentialSession.startSession()](https://developer.apple.com/documentation/secureelementcredential/credentialsession/startsession%28%29)
- [Accessing and using secure element credentials](https://developer.apple.com/documentation/secureelementcredential/accessing-and-using-secure-element-credentials)
- [Secure Element Credential entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.secure-element-credential)
- [Default contactless app entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.secure-element-credential.default-contactless-app)
- [NFC & SE Platform for secure contactless transactions](https://developer.apple.com/support/nfc-se-platform/)
- [objc2 generated framework catalog](https://github.com/madsmtm/objc2/blob/main/crates/objc2/src/topics/about_generated/list_unsupported.md)

## Evidence and checks

- Read-only SDK inspection: `xcode-select -p` returned `/Applications/Xcode.app/Contents/Developer`; `xcodebuild -version` returned Xcode 26.6, build `17F113`.
- Inspected `iPhoneOS.sdk/System/Library/Frameworks/SecureElementCredential.framework/Modules/SecureElementCredential.swiftmodule/arm64e-apple-ios.swiftinterface`, `SecureElementCredential.tbd`, `_SecureElementCredential_UIKit.framework/Modules/_SecureElementCredential_UIKit.swiftmodule/arm64e-apple-ios.swiftinterface`, and `_SecureElementCredential_SwiftUI.framework/Modules/_SecureElementCredential_SwiftUI.swiftmodule/arm64e-apple-ios.swiftinterface`. The core framework tree contains no public `.h` headers or module map.
- Read-only binding/source searches found the `objc2` 0.6.5 catalog entry `SecureElementCredential | Swift-only`; no generated local Rust binding or repository Rust/C/Objective-C implementation was found.
- Apple primary docs confirm `isEligible` semantics, the `startSession()` entitlement and user-consent requirements, ABR setup, current region/device availability, and foreground transaction policy.
- No tests, builds, link probes, entitlement requests, credential reads, or runtime probes ran.
