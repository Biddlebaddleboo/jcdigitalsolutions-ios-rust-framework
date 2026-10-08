# PLAN_VALIDATION_IOS_SHARING.md — Workstream G7: iOS Sharing CI Gates

## Objective

Add persistent iOS device and simulator compile/lint gates for `ios-sharing`

Record that these checks do not prove live pasteboard privacy UX or share UI behavior

## Dependencies

- G1 shared CI is integrated
- B6 `ios-sharing` and `docs/ios/sharing.md` are integrated
- D5 and D6 `framework-sharing` contracts are integrated

## Read first

- `PLAN_VALIDATION.md`
- `PLAN_IOS_CLIPBOARD.md`
- `PLAN_CAPABILITIES_CLIPBOARD.md`
- `PLAN_CAPABILITIES_SHARE.md`
- `docs/VALIDATION.md`
- `docs/ios/sharing.md`
- `.github/workflows/ci.yml`

## Write scope

- `.github/workflows/ci.yml`
- `docs/VALIDATION.md`

Do not edit iOS backend code, portable contracts, root Cargo config, `Cargo.lock`, capability status, or runtime behavior. Do not add live pasteboard or share UI actions, signing, or device execution claims

## Required CI

- Add locked `cargo check` gates for `ios-sharing` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Add `cargo clippy --all-targets -- -D warnings` gates for the same package and targets
- Run the gates on macOS runners with both Rust targets and Xcode SDKs
- Update `docs/VALIDATION.md` with exact commands and the limit: compile/lint only, no live pasteboard, privacy prompt, share UI, activity result, or callback/drop race proof

## Validation and handoff

- Run all four target-specific commands
- Parse `.github/workflows/ci.yml`, run `cargo xtask docs-check`, and run `git diff --check`
- Report changed files, commit SHA, exact commands/results, deviations, and open assumptions. Do not push
