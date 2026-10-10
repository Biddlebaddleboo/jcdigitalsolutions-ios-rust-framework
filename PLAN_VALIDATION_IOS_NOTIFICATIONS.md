# PLAN_VALIDATION_IOS_NOTIFICATIONS.md — G5: Local Notification and Response Gates

## Status and scope

G5's CI device/simulator static gates are integrated for B4 `ios-notifications` and B12 `ios-notification-responses`. This workstream owns their validation wiring and evidence only, not the native APIs or R1/R2 installed tooling. Start at `PLAN_IOS_NOTIFICATIONS.md`, `PLAN_IOS_NOTIFICATION_RESPONSES.md`, `.github/workflows/ci.yml`, and `docs/VALIDATION.md`.

## Existing validation evidence and mandatory gates

- The macOS workflow historically contains all eight locked target gates: `cargo check` for **both crates** on `aarch64-apple-ios` and `aarch64-apple-ios-sim`, plus `cargo clippy --all-targets -- -D warnings` for **both crates** on **both targets**. Check they are still present and re-run affected gates.
- The CI runner must install both Apple Rust targets before these gates. Preserve package boundaries; notification response bridging cannot be treated as covered by merely building the notifications crate.
- CI YAML parsing with Ruby (`YAML.parse_file`), `cargo xtask docs-check`, and `git diff --check` were previously successful. Documentation must distinguish compile/lint evidence from notification delivery and response callback execution.
- G5's historical work did not execute tests or launch an app; the earlier checks were local, not proof that a CI workflow run passed. Xcode 26.6/SDK 26.5 does not qualify Xcode 27.x.

## Installed tool integration

Follow `docs/SHARED_TOOLING.md`. These crates are not registered as the four current `ios-rust-validate` pilots. Continue the exact CI gates above until equivalent declarative schema-v1 profiles exist and have positive/negative, skip/failure, two-crate and both-target parity. New API-specific Python adapters may add bounded deterministic assertions but cannot substitute for compiler, binary or user-visible runtime proof. Do not edit historical shared engine source or reopen completed R1/R2 work; report suspected engine defects using `BUG_REPORT_*.md`.

## Explicit runtime exclusions

No request for notification permission, user authorization, scheduling, background delivery, displayed notification, response selection, callback execution or physical-device/simulator behavior was exercised by G5. Those require separately scoped host app/runtime evidence; do not upgrade capability status or imply live functionality from a compiler PASS.

## Handoff

Scope writes to validation CI/docs and this workstream's plan; do not change notification API semantics, platform implementation, manifest or unrelated jobs. Record current CI YAML check, exact checked Rust targets and crates, Clippy results, skipped gates, docs checks, changed paths and commit SHA.