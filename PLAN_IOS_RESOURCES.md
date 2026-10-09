# PLAN_IOS_RESOURCES.md — Workstream B10: iOS Packaged Resources Backend

## Status

B10 is integrated as `ios-resources` for exact, synchronous reads of ordinary files in the main
application bundle. Locked device and simulator checks plus strict all-target Clippy pass on the
recorded Xcode 26.6 host. The shared `PLAN.md` requires Xcode 27.x, so the required toolchain
baseline remains unmet. D7's follow-up path-validation change rejects Windows drive-prefixed paths.
These are compile/lint checks only; no tests were added or run, and no live bundle lookup or
packaging behavior is claimed.

## Objective

Implement an iOS backend for the read-only packaged-resource contract from D7. Support ordinary files in the main application bundle only; do not claim asset-catalog, localization, arbitrary URL, or resource enumeration support.

## Dependencies

- Foundation A and iOS runtime B are integrated
- D7 `framework-resources` and its exact resource-path/backend contract are integrated

## Write scope

- `platform/ios/ios-resources/**`
- `docs/ios/resources.md`

Do not edit the portable D7 contract, root workspace configuration, `Cargo.lock`, capability manifest, shared capability summary, or unrelated iOS crates. The orchestrator owns dependency/lockfile and shared capability-index reconciliation. Do not add Swift source, signing, or build-system resource generation.

## Backend requirements

- Add an independently scoped `ios-resources` crate implementing the D7 backend for the main application bundle.
- Accept only D7's validated relative resource path; never expose an absolute bundle path or arbitrary native URL through the portable API.
- Read ordinary packaged files synchronously into caller-owned bytes and preserve UTF-8/error mapping from D7.
- Resolve only exact resource names; do not normalize paths, perform localized lookup, enumerate directories, decode formats, or access asset catalogs.
- Document blocking/copy behavior, missing-resource mapping, main-bundle scope, and any path/security boundary.
- Derive the public API availability floor from the inspected SDK declarations. Record the actual SDK target minimum separately; do not infer a shared deployment target.
- Document framework, permission, and entitlement requirements only when verified; do not claim live app behavior unless exercised.

## Validation and handoff

- Run device and simulator `cargo check --locked` and strict all-target Clippy for `ios-resources`.
- Run `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`.
- Do not add or run tests in this slice.
- Report crate API, minimum-OS evidence, exact commands/results, changed files, deviations, and unresolved assumptions. Do not edit the shared capability index; the orchestrator updates it with D7 integration.

## Verification evidence — 2026-10-08

The line-by-line plan/backend/guide audit found no acceptance mismatch. The backend uses only
`ResourcePath`, exact UTF-8 path text, `NSBundle.mainBundle()`, nil localization, and
`NSData.dataWithContentsOfURL:options:error:`. It returns caller-owned bytes, maps missing bundle
URLs and Cocoa no-such-file codes to `NotFound`, preserves a nonzero `NSError.code` when it fits
`i32`, and leaves other Foundation errors as `Platform`. The guide accurately limits support to
global ordinary files in the main bundle and makes no symlink-containment, live packaging,
localization, asset-catalog, or external-file-access claim. It states that bundle-local reads need
no app permission or entitlement.

On Xcode 26.6 (build 17F113), `xcrun --sdk iphoneos --show-sdk-path` selected
`iPhoneOS26.5.sdk`; `xcrun --sdk iphonesimulator --show-sdk-path` selected
`iPhoneSimulator26.5.sdk`. The iOS 26.5 `NSBundle.h` declaration marks
`URLForResource:withExtension:subdirectory:localization:` available from iOS 4.0 and documents
that nil localization selects global resources. `NSData.h` declares
`dataWithContentsOfURL:options:error:` without a higher availability annotation. The iOS SDK's
`SDKSettings.json` reports minimum deployment target 12.0, recommended 15.0, and default 26.5;
these SDK values do not set a repository deployment target. The shared `PLAN.md` requires Xcode
27.x, so the planned toolchain baseline remains open.

The following commands passed in the current worktree:

- `cargo check --locked -p ios-resources --target aarch64-apple-ios`
- `cargo check --locked -p ios-resources --target aarch64-apple-ios-sim`
- `cargo clippy --locked -p ios-resources --all-targets --target aarch64-apple-ios -- -D warnings`
- `cargo clippy --locked -p ios-resources --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo xtask docs-check`
- `cargo xtask zero-swift-source`
- `git diff --check`

No tests, app launch, live bundle-resource lookup, package/runtime validation, or filesystem
symlink-containment validation were performed. These results provide compile/lint and static SDK
evidence only; Xcode 27.x and runtime packaging evidence remain unresolved.
