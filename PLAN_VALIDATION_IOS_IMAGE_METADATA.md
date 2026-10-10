# PLAN_VALIDATION_IOS_IMAGE_METADATA.md — Workstream G20: ImageIO Metadata Validation Gate

## Shared-tooling execution (current)

The R1/R2 PATH executables are complete; use `docs/SHARED_TOOLING.md` and schema-v1 specifications rather than new build/validation machinery. This workstream has **no existing registered pilot** in `tools/validation/specs/validation-v1.json`. Retain its recorded local commands and unique ImageIO binary linkage and metadata invariants tests. An executor may declare a new validation profile and optional focused Python adapter; remove or replace existing checks only after equivalent positive, negative, cross-target and failure/skip parity is established. Do not claim an unregistered `ios-rust-validate --capability` run, infer runtime proof from static checks, or retrieve shared engine source. Escalate confirmed engine defects with `BUG_REPORT_*.md`.

## Objective

Provide package-local CI commands and an import audit for the integrated D21/B26 metadata-only slice. This plan does not edit shared CI or global validation docs.

## Dependencies

- D21 `framework-image` and B26 `ios-image-io` are integrated
- The root integration has added the `objc2-image-io` 0.3.2 dependency graph to `Cargo.lock`
- Both iOS Rust targets and Xcode SDKs are available on a macOS host

## Write scope

- `platform/ios/ios-image-io/check-ios.sh`
- `platform/ios/ios-image-io/check-link-imports.sh`

Do not edit CI workflows, `docs/VALIDATION.md`, root Cargo files/lockfile, aggregate plans, status JSON, docs indexes, `tools/xtask`, or capability/backend source.

## Required gates

- Run locked `cargo check -p ios-image-io --target aarch64-apple-ios`
- Run locked `cargo check -p ios-image-io --target aarch64-apple-ios-sim`
- Run locked `cargo clippy -p ios-image-io --all-targets --target aarch64-apple-ios -- -D warnings`
- Run locked `cargo clippy -p ios-image-io --all-targets --target aarch64-apple-ios-sim -- -D warnings`
- Build a temporary static-library consumer outside the repo for each target; use `nm -u` to require the selected ImageIO/CoreFoundation imports and reject CoreImage, CoreGraphics raster-creation, UIKit, Swift runtime, and unrelated capability symbols/features
- Keep all temporary consumer source and artifacts outside the repository

## Evidence limits

These checks prove source compilation/lint and link-symbol reachability only. They do not run ImageIO against a live image, prove format coverage, metadata parity across OS releases, memory/latency bounds, or prove that ImageIO internals never parse/decode data.

## Handoff

Report exact commands/results, changed files, commit SHA, and any integration gap. Root may wire the package script into shared CI after lockfile reconciliation.
