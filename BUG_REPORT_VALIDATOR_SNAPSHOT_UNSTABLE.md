# Tooling bug: clean-tree validation snapshot is unstable

- State: suspected tooling defect
- Affected tool: ios-rust-validate
- Tool version, release ID, full source SHA: 0.1.0; `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`; `350971094cc3b77191cd6b3d07e01af04d71943d`
- Binary SHA-256: `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`; macOS 26.6.2; Xcode 26.6 / iPhoneOS SDK 26.5; `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: `framework-bluetooth` and `ios-bluetooth`; registered B570 central-connection validation

## Minimal reproduction

From a clean checkout at committed framework revision `faa8fe972f123cd5ca3f8979bad28a38629208f`, with the valid schema-v1 repository spec:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability framework-bluetooth --format json
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-bluetooth --format json
```

Both commands reproduce the same failure before any selected validation gate runs.

## Expected behavior

The selected capability profile should be captured and its registered gates should run. `docs/SHARED_TOOLING.md` states that the newer validator snapshots source and supports capability validation, and that local snapshot validation does not require a Git commit. This reproduction uses a committed, clean worktree.

## Actual behavior

Both commands exit 1 and emit:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

No test, compile, Clippy, documentation, ABI/link, or source-guard gate ran. This is `ERROR-BLOCKED`, not validation success.

## Troubleshooting already performed

- Confirmed `command -v ios-rust-validate` resolves to the selected local executable; recorded its `--version --format json`, `--help`, and SHA-256.
- Confirmed `--explain ios-bluetooth` parses the registered profile and lists its selected host, device, Simulator, shell, link-fixture, source, and documentation gates.
- Parsed `tools/validation/specs/validation-v1.json` and `docs/capabilities/capability-status.json` successfully and ran `git diff --check`.
- Reproduced first with a dirty integration worktree, then again after committing the integration metadata and confirming the root worktree was clean.
- Ran a second, minimal known profile (`framework-bluetooth`) from the same clean committed worktree; it returned the identical pre-gate error.
- Retried with `--diagnostics detailed`; the structured error was unchanged.
- Did not inspect or modify shared validator engine source, pinned archives, manifests, installer, or CI tooling workflow.

## Impact

Registered validator qualification for B570 is blocked in this environment, including otherwise independent `framework-bluetooth` and `ios-bluetooth` profiles. Direct Cargo or shell checks do not replace the required registered gate. Previously completed workstreams retain only their own snapshot-tagged validation results; this report does not upgrade or repeat those results. New work may continue only when it is genuinely independent and has its own verified required gates.

## Evidence and attachments

The reproduction output above is sanitized. The captured root revision was `faa8fe972f123cd5ca3f8979bad28a38629208f`; the root worktree was clean for the second reproduction. No credentials or user data are included.

## Recommended maintenance investigation

In a separately authorized tooling-maintenance session, inspect the immutable source matching source SHA `350971094cc3b77191cd6b3d07e01af04d71943d` and determine why capture reports source changes for a clean committed worktree. Confirm the exact trigger with a minimal engine test before proposing a fix; no implementation cause is inferred here.
