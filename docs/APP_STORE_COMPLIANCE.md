# App Store Compliance

## Hard requirement

App Store compliance is an architectural requirement, not a release-time cleanup task.

The framework must use supported public Apple APIs and capabilities for shipping functionality.

## Prohibited implementation shortcuts

Do not ship code that relies on:

- private frameworks;
- private symbols/selectors;
- undocumented daemon/service protocols;
- private entitlements;
- entitlement bypasses;
- reverse-engineered private APIs;
- dynamically downloaded executable behavior contrary to App Store policy;
- tricks intended to evade review or API restrictions.

## Low-level does not mean private

Using Rust FFI, objc2, C, Objective-C ABI, public Darwin APIs, public CoreFoundation APIs, or public Swift ABI entrypoints can be acceptable when the underlying interface and usage are public and permitted.

The framework must not infer that a symbol is acceptable merely because it can be linked or called.

## Feature compliance record

Any unusual platform feature should document:

- Apple framework/API used;
- public documentation source;
- minimum OS version;
- required entitlement/capability;
- whether that entitlement is generally available;
- intended API purpose;
- whether runtime-downloaded code/data is involved;
- fallback behavior;
- review risks or restrictions.

## Swift ABI

Direct Rust interoperability with a public Swift-only Apple API is acceptable only when it uses supported public API surface and does not depend on private symbols or implementation details that make shipping fragile or noncompliant.

## Entitlements

Do not request restricted entitlements merely to reproduce behavior unavailable to normal third-party apps.

If a feature requires an entitlement unavailable to ordinary App Store apps, mark the feature unsupported for the general framework unless project scope explicitly changes.

## Review discipline

When Apple changes review rules or platform requirements, update this document and any affected implementation before making compatibility claims.
