# PLAN_VALIDATION_IOS_NOTIFICATIONS.md — Workstream G5: iOS Notification Target Gates

## Objective

Keep the iOS local-notification backend's device and simulator build contracts in macOS CI.

## Dependencies

- `PLAN_IOS_NOTIFICATIONS.md` and `ios-notifications` are integrated
- The existing CI target installation and validation docs are available
- Add no dependency

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not alter notification API semantics, iOS backend code, the global capability manifest, or unrelated CI jobs.

## Required gates

- On macOS, run locked `cargo check` for `ios-notifications` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- On macOS, run Clippy with `-D warnings` for all targets of `ios-notifications` on both targets
- Keep host workspace tests as the only automated runtime-test claim; do not add live permission-prompt or notification-delivery tests
- Update validation docs to separate compile/lint gates from device/simulator permission and delivery evidence

## Validation and handoff

- Verify workflow YAML and run the same cargo commands locally where the installed Apple SDK permits
- Run `cargo xtask docs-check` and `git diff --check`
- Report changed files, commit SHA, checks, deviations, and unresolved assumptions
