# PLAN_VALIDATION_IOS_NFC.md — G31: Core NFC reader and NDEF-read gates

## Objective and implementation scope
Gate the narrow D32/B37 Core NFC reader-support query and B573 one-shot NDEF reads. Start with `PLAN_CAPABILITIES_NFC.md`, `framework-nfc`, `ios-nfc`, and their registered local checks. The target Rust components and Apple SDK must be available.

## Ownership and non-changes
This G31 slice owns this plan and package-local source/guides only when completing D32/B37. CI workflow, `docs/VALIDATION.md`, workspace configuration, capability matrix, indexes and unrelated code remain orchestrator/integration-owned. Lockfile changes are limited to dependencies actually required for the scoped feature. No change to NFC runtime behavior is authorized solely for validation migration.

## Installed-tooling boundary
G31 originally had no registered profile in `tools/validation/specs/validation-v1.json`. B573 now registers `framework-nfc` for the portable contract and dependency graph, and `ios-nfc` for the device/Simulator builds, lifecycle regressions, and release link/import audit. Use the installed validator profiles as acceptance gates; retain the specialized release audit and its negative assertions.

## Required validation
On macOS with both iOS targets, use the registered validator profiles as the acceptance gates:
```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability framework-nfc
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-nfc
```
The `framework-nfc` gate covers the portable contract and dependency graph. The `ios-nfc` gate covers host/device/Simulator compilation, shell syntax, source regressions, and the Release link/import audit. The audit verifies the `objc2-core-nfc` dependency for device and Simulator, the exact `CoreNFC` and `Foundation` framework set, runtime imports, prohibited symbols, and deployment floors. Additionally assert no shipping `.swift` source. A `cargo check` alone cannot satisfy the Release link/import requirement.

## Evidence limits and handoff
Passing these gates proves only the checks actually executed. It does not prove hardware availability, provisioning or `Info.plist`, NFC prompts, session startup, tag scanning/reads/writes, background reading or simulator NFC behavior. Report exact commands, artifact/target, imports, skipped gates, original-regression coverage and commit SHA. Do not retrieve historical engine source; escalate engine defects through `BUG_REPORT_*.md`.

## Hosted CI requalification

[GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431) completed successfully at source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`. In both `Rust checks (macos-15)` and `Rust checks (xcode-27)`, the run passed `Check portable NFC contract without default features`, `Test portable NFC contract` (one unit test passed), `Clippy portable NFC contract`, `Check iOS Core NFC device target`, `Check iOS Core NFC Simulator target`, `Clippy iOS Core NFC device target`, and `Clippy iOS Core NFC Simulator target`. Ubuntu passed the portable NFC checks; its Apple-target steps were skipped. The Xcode 27 job recorded Xcode 27.0 build `27A266a`, iPhoneOS SDK 27.0, and iPhoneSimulator SDK 27.0. The NFC package/workflow and shared Cargo inputs for these gates are unchanged through current base `f7a6d0cd98d9a23ecd718fbc98751f46c048a204`.


## Package-local Release link/import audit

`sh platform/ios/ios-nfc/check-link-imports.sh` passed locally with Rust 1.94.1, Xcode 26.6 build `17F113`, iPhoneOS SDK 26.5, and iPhoneSimulator SDK 26.5. It built Release link probes for `aarch64-apple-ios` and `aarch64-apple-ios-sim` at `target/<target>/release/examples/ios_nfc_link_import_probe`. For each target, the selected dependency was `objc2-core-nfc v0.3.2`; the portable `framework-nfc` dependency graph excluded `objc2-core-nfc`; the linked framework set was exactly `CoreNFC` and `Foundation`; no unrelated capability crate, Swift/Python binding dependency, Swift/Python runtime library, or Swift/Python runtime symbol was found. The probe binaries were linked and inspected, not executed. Xcode 26.6 is below the repository's Xcode 27.x baseline, so this is local audit evidence only.

Hosted evidence for the Release audit remains pending: run 38075483431 had no linked Core NFC artifact audit. The current workflow remains manual-dispatch only; no hosted job was launched for this change. The hosted run also has no `git diff --check` gate for this slice; local diff checks do not imply hosted evidence. Compile, lint, link/import, and unit-test results do not establish NFC hardware availability, provisioning, prompts, session lifecycle, tag operations, or simulator/device runtime behavior.

## B573 registered validator results — 2026-10-12

The pinned validator at `/Users/john/Projects/jcdigitalsolutions-ios-rust-framework/target/ios-rust-tools/bin/ios-rust-validate` reported release `ios-rust-tools-0.1.0+2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`, source SHA `2289e6a73257b696f6ae5ecd61ee20fd16ab8b37`.

- `--capability framework-nfc`: 18 PASS, 0 FAIL, 0 BLOCKED, 2 SKIPPED.
- `--capability ios-nfc`: 28 PASS, 0 FAIL, 0 BLOCKED, 2 SKIPPED.
- Both profiles skipped Simulator runtime execution and physical-device proof. These results establish host/target compilation, static/link checks and release artifact inspection only; they do not establish NFC runtime behavior.
- The Release link/import audit verified the linked device deployment floor as iOS 11.0 and Simulator floor as iOS 14.0. It inspected artifacts for the expected framework/import set; the probes were not executed.
