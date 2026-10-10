# PLAN_VALIDATION_IOS_BACKGROUND_TASKS.md — G19: App Refresh Task Gates

## Status and objective

G19's D20/B25 static validation gates were previously implemented. The workstream does **not** establish a scheduled/live BackgroundTasks invocation. Preserve the original compiler, linker, documentation and negative-import requirements while using completed R1/R2 tooling for routine execution. See `PLAN_CAPABILITIES_BACKGROUND_TASKS.md`, `PLAN_IOS_BACKGROUND_TASKS.md`, `docs/SHARED_TOOLING.md`.

## Implementation scope and verified current boundary

Start at `crates/framework-background/Cargo.toml`, `platform/ios/ios-background-tasks/Cargo.toml`, `platform/ios/ios-background-tasks/check-link-imports.sh`, and the current `tools/validation/specs/validation-v1.json`. The capability is not one of the four registered validator pilots at the inspected baseline. Do not claim that `ios-rust-validate --capability ios-background-tasks` works until a matching schema-v1 entry is committed and verified.

## Required static evidence

- Locked `framework-background --no-default-features` check and strict all-target Clippy; no_std/feature isolation must survive.
- Locked `ios-background-tasks` check and strict all-target Clippy on both `aarch64-apple-ios` and `aarch64-apple-ios-sim`.
- Formatting for both manifests, Rustdoc for portable and native crates, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`.
- Execute `sh platform/ios/ios-background-tasks/check-link-imports.sh` on supported macOS. Preserve linked device/simulator probe checks for direct framework import allowlist, selected BackgroundTasks symbols, exclusion of Swift/runtime and unrelated frameworks, and the declared OS floor.
- Historical proof used Rust 1.94.1 and Xcode 26.6 / SDK 26.5; re-run under current installed host and separately qualify Xcode 27.x before making that claim.

## Shared-tool migration contract

For new scoped work, prefer declarative `tools/validation/specs/schema-v1.json` rules and optional bounded Python adapters per `docs/SHARED_TOOLING.md`; migrate repeated Cargo formatting/lint/docs only after the new profile passes positive and negative regression fixtures and retains all device/simulator link checks. Do not change the installed engine, revive R1/R2 plans, remove the focused link script without equivalent coverage, or silently treat SKIPPED/ERROR-BLOCKED as PASS. An engine defect becomes a sanitized committed `BUG_REPORT_*.md` in a separately authorized implementation session.

## Explicitly unverified runtime scope

No app launch, Info.plist scheduling configuration, real schedule request, delivered callback, expiration handler, task completion, device run, or simulator run has been established by these static gates. Record runtime evidence separately; never infer scheduler behavior or timing from successful checks and linked probes.

## Validation and handoff

Run the existing checks above until parity-tested registration exists; inspect `ios-rust-validate --list`/`--explain ID` before using any new profile. Report exact commands and target, Rust/Xcode/SDK versions, direct imports/minimum OS, skipped gates, original regression protection, changed paths and commit SHA. Keep unexecuted device/OS qualification explicitly open.