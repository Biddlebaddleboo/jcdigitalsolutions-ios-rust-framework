# PLAN_VALIDATION_IOS_CONTACTS.md — Workstream G23: iOS Contacts Package Gates

## Shared-tooling execution (current)

R1/R2 tooling is complete and installed on PATH; consult `docs/SHARED_TOOLING.md`. This capability is **not registered** in the current four-pilot `tools/validation/specs/validation-v1.json`; do not claim an `ios-rust-validate --capability ios-contacts` PASS. Retain the existing focused shell/Cargo checks below, especially portable Contacts tests, permissions and ABI. When implementing new acceptance coverage, add schema-v1 declarative rules and, only if necessary, bounded Python adapters; replace repeated formatting/lint orchestration **only after** matching the original positive and negative gates. Existing recorded commands remain requirements/evidence as labeled. Never inspect or change the historical shared engine source; report engine defects in a sanitized `BUG_REPORT_*.md`.

## Objective

Record and provide package-local compile/lint gates for the D24 portable Contacts contract and B29 iOS authorization backend. Central CI integration remains an orchestrator-owned follow-up.

## Dependencies

- D24 `framework-contacts` and B29 `ios-contacts` are present
- The integrated workspace lockfile resolves `objc2-contacts` 0.3.2
- Both Apple Rust targets and their Xcode SDKs are installed on the macOS validation host

## Write scope

- `PLAN_VALIDATION_IOS_CONTACTS.md`
- Package-local scripts under `platform/ios/ios-contacts/scripts/**`
- `docs/ios/contacts.md`

Do not edit `.github/workflows/ci.yml`, root workspace files or lockfile, canonical capability data, aggregate plans/indexes, `tools/xtask`, or runtime implementation.

## Required gates

- Portable contract: `cargo test -p framework-contacts` and `cargo check -p framework-contacts --no-default-features`
- iOS device and simulator: locked `cargo check -p ios-contacts` and strict Clippy with `-D warnings` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run formatting, docs/link checks, dependency-feature review, and `git diff --check`
- State that compile/lint and package-feature evidence does not exercise permission UI, user consent, Limited-selection UI, Settings changes, or contact fetch/data access
- Do not add live prompt, signing, simulator UI, or device-consent automation

## Handoff

Report exact commands/results, the lockfile prerequisite for locked target gates, API and Limited-status availability facts, changed files, deviations, and unresolved assumptions. CI workflow and shared validation-index integration are separate owner work.
