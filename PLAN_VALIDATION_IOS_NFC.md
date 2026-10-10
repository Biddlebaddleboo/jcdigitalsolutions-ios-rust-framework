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
git diff --check
```
Additionally assert no shipping `.swift` source; inspect a Release-linked iOS artifact for required `CoreNFC` and `Foundation` imports, the `objc2-core-nfc` dependency boundary, and absence of unrelated capability frameworks or Swift-runtime imports. A `cargo check` alone cannot satisfy the Release link/import requirement. Preserve documented target/deployment floors and public API guarantees.

## Evidence limits and handoff
Passing these gates proves only the checks actually executed. It does not prove hardware availability, provisioning or `Info.plist`, NFC prompts, session startup, tag scanning/reads/writes, background reading or simulator NFC behavior. Report exact commands, artifact/target, imports, skipped gates, original-regression coverage and commit SHA. Do not retrieve historical engine source; escalate engine defects through `BUG_REPORT_*.md`.