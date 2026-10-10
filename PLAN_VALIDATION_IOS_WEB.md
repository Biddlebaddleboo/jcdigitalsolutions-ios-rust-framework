# PLAN_VALIDATION_IOS_WEB.md — Workstream G27: Web View Package Gates

## Shared-tooling execution (current)

The R1/R2 PATH executables are complete; use `docs/SHARED_TOOLING.md` and schema-v1 specifications rather than new build/validation machinery. This workstream has **no existing registered pilot** in `tools/validation/specs/validation-v1.json`. Retain its recorded local commands and unique WebKit ownership, feature guards and runtime boundaries tests. An executor may declare a new validation profile and optional focused Python adapter; remove or replace existing checks only after equivalent positive, negative, cross-target and failure/skip parity is established. Do not claim an unregistered `ios-rust-validate --capability` run, infer runtime proof from static checks, or retrieve shared engine source. Escalate confirmed engine defects with `BUG_REPORT_*.md`.

## Objective

Add package-local validation gates for the integrated `framework-web` contract and iOS `ios-web`
adapter without changing shared CI or aggregate validation documents.

## Dependencies

- D28 `framework-web` and its package-local gate are integrated
- B33 `ios-web` and its package-local gate are integrated
- Root lockfile includes the exact `objc2-web-kit 0.3.2` dependency graph
- The host has the Rust targets `aarch64-apple-ios` and `aarch64-apple-ios-sim` plus Xcode SDKs

## Write scope

- `PLAN_VALIDATION_IOS_WEB.md`
- `crates/framework-web/check.sh`
- `platform/ios/ios-web/check.sh`
- `platform/ios/ios-web/check-surface.sh`

Do not edit shared CI, `PLAN_VALIDATION.md`, docs indexes, root workspace/dependency declarations,
`Cargo.lock`, capability matrix, or backend runtime behavior. The orchestrator owns shared CI and
validation-document integration.

## Required local gates

- `crates/framework-web/check.sh` runs locked no-default-feature check, tests, and strict
  all-target Clippy.
- `platform/ios/ios-web/check.sh` runs locked device and simulator checks and strict all-target
  Clippy.
- `platform/ios/ios-web/check-surface.sh` rejects source references to file/HTML/data loading,
  JavaScript evaluation, script-message handlers, `UIWebView`, or `WKUIDelegate`.
- The implementation executor also runs formatting, docs-check, zero-Swift-source, and
  `git diff --check`.
- Compile/lint evidence does not prove app launch, UI visibility, page loading, network behavior,
  privacy prompt behavior, or runtime navigation policy.

## Status

Package-local scripts are implemented and passed in isolated worktree
`/Users/john/Projects/.worktrees/jcdig-webkit-d28` on `workstream/capabilities-webkit-d28`:
`crates/framework-web/check.sh`, `platform/ios/ios-web/check.sh`, and
`platform/ios/ios-web/check-surface.sh`. Docs, formatting, zero-Swift-source, and diff checks also
passed. No shared CI or aggregate validation file was changed.

The locked scripts ran against the temporary resolved lock containing `objc2-web-kit 0.3.2`; that
lock will be restored before handoff. Root lockfile integration is required before the scripts can
run with `--locked` in the integrated checkout. The gates establish compile/lint/surface evidence
only, not iOS runtime, UI, network, or page-policy behavior.
