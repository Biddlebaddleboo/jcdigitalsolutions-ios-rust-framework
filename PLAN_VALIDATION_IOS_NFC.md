# PLAN_VALIDATION_IOS_NFC.md — G31: Core NFC snapshot gates

## Objective and implementation scope
Gate the narrow D32/B37 Core NFC reader-support query—not NFC session lifecycle or tag operations. Start with `PLAN_CAPABILITIES_NFC.md`, `PLAN_IOS_NFC.md`, `framework-nfc`, `ios-nfc`, and their existing local link/import checks. The target Rust components and Apple SDK must be available.

## Ownership and non-changes
This G31 slice owns this plan and package-local source/guides only when completing D32/B37. CI workflow, `docs/VALIDATION.md`, workspace configuration, capability matrix, indexes and unrelated code remain orchestrator/integration-owned. Lockfile changes are limited to dependencies actually required for the scoped feature. No change to NFC runtime behavior is authorized solely for validation migration.

## Installed-tooling boundary
The pinned R1/R2 binaries exist on PATH (`docs/SHARED_TOOLING.md`); G31 has **no registered profile** in `tools/validation/specs/validation-v1.json`. Retain all focused commands until a schema-v1 pilot demonstrably covers them. Adding a new profile is an API-specific declarative change, not shared engine work. Built-in check/lint/doc guards should replace duplicated orchestration only after equivalence; keep specialized binary/import and negative assertions.

## Required validation
On the macOS host with both iOS targets:
```sh
cargo fmt --all -- --check
cargo test -p framework-nfc
cargo check -p framework-nfc --no-default-features
cargo check --locked -p ios-nfc --target aarch64-apple-ios
cargo check --locked -p ios-nfc --target aarch64-apple-ios-sim
cargo clippy --locked -p ios-nfc --all-targets --target aarch64-apple-ios -- -D warnings
cargo clippy --locked -p ios-nfc --all-targets --target aarch64-apple-ios-sim -- -D warnings
cargo xtask docs-check
sh platform/ios/ios-nfc/check-link-imports.sh
git diff --check
```
The package-local Release audit asserts no Swift/Python binding dependency or runtime import, verifies the `objc2-core-nfc` dependency for device and Simulator, and requires the linked framework set to contain only `CoreNFC` and `Foundation`. Additionally assert no shipping `.swift` source. A `cargo check` alone cannot satisfy the Release link/import requirement. Preserve documented target/deployment floors and public API guarantees.

## Evidence limits and handoff
Passing these gates proves only the checks actually executed. It does not prove hardware availability, provisioning or `Info.plist`, NFC prompts, session startup, tag scanning/reads/writes, background reading or simulator NFC behavior. Report exact commands, artifact/target, imports, skipped gates, original-regression coverage and commit SHA. Do not retrieve historical engine source; escalate engine defects through `BUG_REPORT_*.md`.

## Hosted CI requalification

[GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. In both `Rust checks (macos-15)` and `Rust checks (xcode-27)`, the run passed `Check portable NFC contract without default features`, `Test portable NFC contract` (one unit test passed), `Clippy portable NFC contract`, `Check iOS Core NFC device target`, `Check iOS Core NFC Simulator target`, `Clippy iOS Core NFC device target`, and `Clippy iOS Core NFC Simulator target`. Ubuntu passed the portable NFC checks; its Apple-target steps were skipped. The Xcode 27 job recorded Xcode 27.0 build `27A266a`, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. The NFC package/workflow and shared Cargo inputs for these gates are unchanged through current base `f7a6d0cd98d9a23ecd718fbc98751f46c048a204`.


## Package-local Release link/import audit

`sh platform/ios/ios-nfc/check-link-imports.sh` passed locally with Rust 1.94.1, Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5, and iPhoneSimulator SDK 26.5. It built Release link probes for `aarch64-apple-ios` and `aarch64-apple-ios-sim` at `target/<target>/release/examples/ios_nfc_link_import_probe`. For each target, the selected dependency was `objc2-core-nfc v0.3.2`; the portable `framework-nfc` dependency graph excluded `objc2-core-nfc`; the linked framework set was exactly `CoreNFC` and `Foundation`; no unrelated capability crate, Swift/Python binding dependency, Swift/Python runtime library, or Swift/Python runtime symbol was found. The probe binaries were linked and inspected, not executed. Xcode 26.6 is below the repository's Xcode 27.x baseline, so this is local audit evidence only.

Hosted evidence for the Release audit remains pending: run 38075483431 had no linked Core NFC artifact audit, and the workflow remains integration-owned. Add a macOS CI step named `Audit iOS Core NFC Release link/imports` with `sh platform/ios/ios-nfc/check-link-imports.sh`, then record a passing hosted run. The hosted run also has no `git diff --check` gate for this slice; `git diff --check` passed locally for this change, without a hosted claim. Compile, lint, link/import, and unit-test results do not establish NFC hardware availability, provisioning, prompts, session lifecycle, tag operations, or simulator/device runtime behavior.
