# PLAN_IOS_TRACKING_AUTHORIZATION.md — Workstream B40: iOS App Tracking Status Backend

## Objective

Implement D35's App Tracking Transparency status contract with the public `ATTrackingManager.trackingAuthorizationStatus` class property on iOS 14 and later. Do not implement the separate authorization request API.

## Dependencies

- D35 `framework-auth` contract
- Installed iOS SDK metadata and Apple primary documentation
- Generated `objc2-app-tracking-transparency` 0.3.2 bindings

## Read first

- `PLAN_CAPABILITIES_PRIVACY_AUTHORIZATION.md`
- `docs/OBJC_INTEROP.md`
- `docs/UNSAFE.md`
- [Apple `trackingAuthorizationStatus`](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/trackingauthorizationstatus)
- [Apple `AuthorizationStatus`](https://developer.apple.com/documentation/apptrackingtransparency/attrackingmanager/authorizationstatus)
- [Apple `NSUserTrackingUsageDescription`](https://developer.apple.com/documentation/bundleresources/information-property-list/nsusertrackingusagedescription)
- [Apple App Tracking Transparency overview](https://developer.apple.com/documentation/apptrackingtransparency)

## Write scope

- `platform/ios/ios-auth/**`
- `docs/ios/tracking-authorization.md`

Do not edit the portable contract, root workspace configuration, capability status manifest, shared docs indexes, CI, aggregate plans, or `tools/xtask`. Keep this worktree isolated from the shared checkout.

## Required implementation

- Use `objc2-app-tracking-transparency` exactly at version 0.3.2 with default features disabled
- Call only the generated `ATTrackingManager::trackingAuthorizationStatus()` getter and map the four documented enum values; map unrecognized raw values to `Unknown`
- Keep the adapter on iOS device and simulator targets; exclude Mac Catalyst
- Require a host iOS deployment target of 14.0 or later, matching SDK metadata
- Keep the generated unsafe call in a private wrapper with an exact safety comment
- Do not enable the binding's `block2` feature; do not invoke `requestTrackingAuthorizationWithCompletionHandler:`, create `ATTrackingManager`, access IDFA, or perform tracking
- Treat `NSUserTrackingUsageDescription` as required host configuration per Apple's broad App Tracking Transparency API documentation; note that reading the status does not present the authorization prompt
- Do not claim general privacy authorization, account consent, identifier availability, tracking legality, or any unrelated permission
- Do not infer an entitlement requirement not stated by the reviewed Apple documentation

## Validation and handoff

- Run target-specific `cargo check` and strict Clippy for `ios-auth` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run the D35 portable tests and checks
- Inspect binding features, generated signatures, framework link, target imports, status mapping, and absence of request, block, identifier, and tracking paths
- Do not claim live device/simulator status, prompt, or user-choice evidence
- Report changed files, exact commands, API floor, host plist requirement, deviations, and unresolved limits
