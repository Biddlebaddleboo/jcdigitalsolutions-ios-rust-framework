# PLAN_VALIDATION_IOS_CONTACTS.md — Workstream G23: iOS Contacts Package Gates

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
