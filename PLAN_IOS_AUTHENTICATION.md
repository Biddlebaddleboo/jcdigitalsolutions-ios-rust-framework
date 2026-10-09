# PLAN_IOS_AUTHENTICATION.md — Workstream B16: iOS Local Authentication

## Status

B16's line-by-line audit found no policy, API-floor, callback-queue, cancellation, or error-map
mismatch. `BiometricsOnly` maps to the no-passcode-fallback policy; `DeviceOwner` is runtime-gated to
iOS 9. Availability creates a fresh context, and the evaluation callback uses only synchronized
`Arc` state; drop detaches before invalidation and the completion cell wakes after lock release. The
iOS 8.3 `LAErrorDomain` symbol is not referenced: the backend compares against `kLAErrorDomain`,
the SDK macro and generated binding's compile-time string, preserving the iOS 8.0 biometric floor.
No B16 source or guide correction was needed.

On Xcode 26.6 build 17F113 with iPhoneOS and iPhoneSimulator SDK 26.5, these checks pass:

- `cargo check --locked -p ios-auth --target aarch64-apple-ios`
- `cargo check --locked -p ios-auth --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-auth --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-auth --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo doc --locked -p ios-auth --no-deps --target aarch64-apple-ios`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `cargo xtask zero-swift-source`
- `sh platform/ios/ios-auth/check-link-imports.sh`
- `git diff --check`

`cargo tree --locked --target aarch64-apple-ios -e features -p ios-auth` confirms
`objc2-local-authentication` 0.3.2 uses only `LAContext`, `LAPublicDefines`, and `block2` directly;
the probe imports exactly `Foundation`, `LocalAuthentication`, `libSystem.B.dylib`, and
`libobjc.A.dylib` for device and simulator, with no Swift runtime, Security/Keychain, or unrelated
capability imports. No tests were added or run. This host remains below the required Xcode 27.x
baseline, and compile/import evidence does not establish live prompt, biometric sensor, enrollment,
or host-app plist behavior.

## Objective

Implement D12's one-shot local user-presence contract with Apple's public LocalAuthentication `LAContext` API. Keep each operation explicit, caller-owned, statically selected, and isolated from secure storage, passkeys, and general privacy authorization.

## Dependencies

- Foundation A, iOS runtime B, and D12 `framework-auth` are integrated.
- D2 secure storage is not a dependency.
- Inspect the active generated bindings and iOS SDK headers before selecting APIs, features, or stating the effective API floor.
- Use the smallest supported `objc2-local-authentication` feature set if its public bindings express the required callback and ownership contract. Record its dependency rationale and preserve a narrow replacement seam.

## Write scope

- `platform/ios/ios-auth/**`
- `docs/ios/authentication.md`

Do not edit `framework-auth`, root workspace configuration, `Cargo.lock`, shared capability status, shared documentation indexes, CI, Swift ABI, C bindings, Keychain paths, or other capability families. The orchestrator owns shared workspace/lockfile/index integration.

## Required implementation

- Implement `AuthenticationBackend` with policy-specific synchronous availability and one explicit asynchronous operation; do not add a global context, registry, executor, or process initialization.
- Map `AuthenticationPolicy::BiometricsOnly` to Apple's biometrics-only policy and `AuthenticationPolicy::DeviceOwner` to the policy that permits the local device credential. Verify both symbols and availability from the active SDK.
- Query availability with a fresh `LAContext` for each call; do not cache `canEvaluatePolicy` results because enrollment, lockout, and passcode state can change. Never call `canEvaluatePolicy` from the `evaluatePolicy` reply block; Apple warns this can deadlock.
- The inspected Xcode 26.6 / iOS 26.5 SDK marks `LAContext`, biometrics-only policy evaluation, and `evaluatePolicy` as iOS 8.0 APIs, while `DeviceOwnerAuthentication` and `invalidate` are iOS 9.0 APIs. Gate the latter policy and cancellation path by runtime availability; document the effective per-policy floors from the selected bindings and headers.
- Create a fresh `LAContext` for each authentication request, copy the borrowed reason only as needed by native callback state, and start evaluation on the returned future's first poll.
- Keep `LAContext` owned by the future/backend operation until terminal completion or drop. On drop, invalidate the context where supported, suppress the Rust result, and safely ignore a callback racing cancellation.
- Apple's evaluation callback uses an unspecified private queue. Callback state and waker access must therefore be synchronized and thread-safe; never access `Rc`, `RefCell`, or thread-bound state from the callback queue. Wake the future only after releasing locks.
- Ensure callback completion is once-only and releases captured state once. Do not capture the context in a way that creates a retain cycle.
- Map recognized user, app, and system cancellation to `ErrorKind::Cancelled`; map invalid policy/reason and unsupported policy to stable framework errors; preserve nonzero `LAError` codes for other native failures. `LAErrorDomain` is available from iOS 8.3 in the inspected SDK, so do not assume its symbol exists across the full iOS 8.0 biometrics floor; define and document a safe older-OS fallback. Document every code mapping and avoid claiming that authentication establishes identity.
- `BiometricsOnly` must not fall back to a passcode. `DeviceOwner` may use biometrics or the local device credential as Apple's policy defines. Do not expose biometric modality, samples, enrollment records, or templates.
- Document `NSFaceIDUsageDescription` for apps that invoke Face ID. Do not claim that Rust code configures host-app plist values, prompts without explicit requests, or adds an entitlement.
- Use public APIs only. Add no Swift source, custom biometric processing, private API, or dependency on Security/Keychain.

## Documentation and evidence

Document policy mapping, availability re-query behavior, API floor evidence, cancellation races, callback queue, error/native-code mapping, `NSFaceIDUsageDescription`, data/privacy limits, and the fact that target checks do not prove prompt or sensor behavior. Cite Apple's `LAContext`, `canEvaluatePolicy`, `evaluatePolicy`, `invalidate`, `LAError`, and `NSFaceIDUsageDescription` references.

## Non-test validation and handoff

- Do not add or run tests for this workstream.
- Run locked iOS device and simulator `cargo check`, strict Clippy, formatting, rustdoc, docs-check, zero-Swift-source, and `git diff --check` where the installed toolchain permits.
- Inspect the feature graph and linked imports for LocalAuthentication and absence of Keychain, Swift runtime, and unrelated capability frameworks.
- Report changed files, exact commands/results, dependency/features, API floor evidence, imports, deviations, and unresolved assumptions. Do not commit or push.
