# PLAN_IOS_HEALTH_AUTHORIZATION.md — Workstream B31: iOS HealthKit Authorization

## Objective

Implement the portable D26 availability and explicit request contract with public iOS HealthKit APIs, with no Swift source and no authorization-grant inference.

## Scope

- `platform/ios/ios-health-authorization/**`
- `docs/ios/health-authorization.md`
- this named subplan

Do not edit the root workspace configuration or lockfile, shared capability manifest/counts, CI, aggregate plans, docs indexes, `tools/xtask`, portable contracts owned by another workstream, or unrelated workstreams. Root owns workspace dependency/lock/index/capability reconciliation after handoff.

## Backend requirements

- Use `HKHealthStore.isHealthDataAvailable` before all other HealthKit calls and check again before a request.
- Use only `requestAuthorizationToShareTypes:readTypes:completion:` and base public `HKObjectType` factories for quantity, category, characteristic, correlation, and workout types.
- Map `HKHealthStore` request completion `Bool` only to request-flow completion/error. Do not call `authorizationStatus(for:)`; do not expose `HKAuthorizationStatus`; do not infer or return read access.
- Keep callback state `Send`, handle the callback's arbitrary background queue, call the user closure at most once, and prevent Rust panic unwind across the Objective-C block boundary.
- Preserve representable nonzero `NSError` codes; map availability false and unknown identifiers to explicit portable errors.
- Do not create a HealthKit store until the availability check passes. Do not add queries, writes, clinical data, history, observer/background APIs, or UI/runtime initialization.

## Metadata and API constraints

Document the `com.apple.developer.healthkit` entitlement and HealthKit capability; conditional `NSHealthShareUsageDescription` and `NSHealthUpdateUsageDescription` requirements; the optional `UIRequiredDeviceCapabilities` `healthkit` entry; and clinical/background keys as out of scope. Record the iOS 8.0 API floor, per-identifier floors, iPadOS 17 availability behavior, enterprise restriction behavior, Simulator sample-data limits, and no live runtime evidence.

## Status and evidence

Implemented in isolated worktree `/private/tmp/d26-healthkit-worktree`. `cargo check -p ios-health-authorization --target aarch64-apple-ios` and the matching `aarch64-apple-ios-sim` target pass on installed Xcode 26.6 / iOS SDK 26.5. These are target checks, not app launch, link, permission-prompt, or device-runtime evidence. No root integration or commit is included in this handoff.
