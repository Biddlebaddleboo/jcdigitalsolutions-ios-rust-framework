# PLAN_IOS_RESOURCES.md — Workstream B10: iOS Packaged Resources Backend

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
