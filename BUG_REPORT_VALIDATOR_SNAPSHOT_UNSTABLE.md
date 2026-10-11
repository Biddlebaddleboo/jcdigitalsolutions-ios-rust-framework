# Tooling bug: snapshot reports source change in a stable worktree

- State: suspected tooling defect
- Affected tool: `ios-rust-validate`
- Tool version, release ID, full source SHA: `0.1.0`; `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`; `350971094cc3b77191cd6b3d07e01af04d71943d`
- Binary SHA-256: `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`; macOS `26.6.2` (`25G83`); iPhoneOS and iPhoneSimulator SDK `26.5`; `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: `ios-app-lifecycle`; changed-file validation of current framework `main`

## Minimal reproduction

From the framework checkout, with no other validator process active and a clean Git worktree:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-app-lifecycle --format human --diagnostics short
```

The command was also attempted with `--format json`, and the `ios-presentation` profile was attempted with `--diagnostics short` and `--diagnostics detailed`.

## Expected behavior

Per `docs/SHARED_TOOLING.md`, the validator captures a stable source snapshot and reports capability check results. A clean, unchanged checkout should not be classified as source mutation.

## Actual behavior

Repeated root-checkout invocations returned the structured error:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

The same `ios-app-lifecycle` command in an isolated worktree completed with `pass=39 fail=0 blocked=0 skipped=2`. Those two skips were runtime checks; this is not evidence that the root-checkout run passed.

## Troubleshooting already performed

- Confirmed installed release, source SHA, binary checksum, Rust host, macOS version, and both Apple SDK versions
- Confirmed `--list` and `--explain ios-app-lifecycle` work and the spec parses as JSON; current spec size is 261581 bytes
- Confirmed the root worktree was clean and no other validator process was active during the repeated root attempts
- Reproduced in more than one root profile; an isolated-worktree run of the minimal profile completed
- Did not inspect or modify validator engine source

## Impact

Root-checkout profile results are blocked and cannot be reported as validation success. Independent host tests and target checks may continue, but any required validator gate must be re-run after a tooling investigation.

## Evidence and attachments

The safe diagnostic above and pass/fail summary are the available evidence. No credentials or user data are included.

## Recommended maintenance investigation

Compare snapshot capture path selection and change detection for the root checkout and the isolated worktree. Determine whether a stable-file read, path alias, or concurrent workspace artifact is being treated as mutation.
