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

## Additional reproduction — 2026-10-11

The same suspected defect also blocks the integrated B545 camera profile. On framework commit `1f4e8aa`, with a clean worktree, this command exited 1 before any gate ran:

```sh
ios-rust-validate --workspace-root "$PWD" \
  --spec "$PWD/tools/validation/specs/validation-v1.json" \
  --capability ios-camera-capture-session --format json
```

It returned the same `snapshot-unstable: source changed during capture` diagnostic. Repeated root-checkout attempts for `ios-camera-capture-session`, one attempt for `ios-audio-session`, and an attempt with detailed diagnostics produced the same error. `--version --format json` still reported release `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`; the installed binary SHA-256 remained `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`. The updated spec parsed, was 260,261 bytes (under the 262,144-byte limit), `--explain ios-camera-capture-session` succeeded, and `generate-reports --check` passed before the update commit. The failed run after that commit used a clean worktree.

The camera profile now has five B545 source guards for the flash API, preflight, unchanged default path, unsupported-mode regression, and link probe. A separate workstream run on earlier source snapshot `4a665e1f9ab882455b3717d44ea6850499a03f97a6be6ce8a4000ca143666506` passed its then-current profile with 265 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED; that profile predates these five guards. The new guards and integrated profile remain unverified. The two earlier skips were Simulator execution and physical-device proof, not passing runtime evidence.

## Additional reproduction — 2026-10-12 (B554)

The same suspected defect blocks the integrated B554 `ios-ui` profile. The B554 implementation commit `ac5fd9d` and registration commit `07fe85e39a4f7887b1b119deac6f7a788c7f646b` are on local `main`. After correcting the initial profile-registration mistakes, the following commands failed before any gate ran on a clean tree at `07fe85e`:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-ui --format human --diagnostics short
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --changed 85de85d --format human --diagnostics short
```

Both returned `error-blocked: snapshot-unstable: source changed during capture`. Six corrected profile attempts failed before checks, including attempts with staged and unstaged integration files and a clean committed tree. `--explain ios-ui` succeeds and shows the corrected registration: nine ABI/link fixtures, two shell syntax checks, ten collection-grid guards, and unchanged device iOS 12.0 / Simulator iOS 14.0 floors. The 62 declared `ios-ui` inputs were byte-stable during a separate two-second hash check, and no validator or build process was active.

The initial integrated profile attempt used a superseded spec snapshot and reported 278 pass, 7 fail, 0 blocked, and 2 skipped. The seven failures were the outdated Foundation feature guard and Rust files mistakenly listed as shell/ABI scripts; those spec errors were corrected. This result is not a pass for the corrected profile.

The unchanged B554 implementation source had 62 passing host tests, device and Simulator compilation/link probes, strict device Clippy, formatting, and a passing collection-grid link probe before integration. Those direct results do not replace the blocked registered profile or establish Simulator/device runtime behavior. The framework docs record this B554-specific impact in `PLAN.md`.

## Additional reproduction — 2026-10-12 (B567)

After B567 added exact authorization-change events to `ios-location-updates`, the latest local validator continued to return `error-blocked: snapshot-unstable: source changed during capture` before running gates. The selected executable is `~/.local/bin/ios-rust-validate`, release/source SHA `350971094cc3b77191cd6b3d07e01af04d71943d`, SHA-256 `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`; help and `--explain ios-location-updates` succeed. The spec is valid at 262090 bytes, below the 262144-byte limit.

The final capability command failed twice on the dirty worktree, once with supported `--watch`, and again on clean committed B567 source `020c8886c705062d87aa60624114804897e5cd1e`. The current source therefore has no registered profile result. An earlier B567 snapshot `95f63f981ba133d0366d3d20f8c37ea7612425c089331fef9268b2b786676c51` reported 46 PASS, 0 FAIL, 0 BLOCKED, and 2 SKIPPED of 48 before the final queue-order test and negative-fixture strengthening; it does not certify the current commit.

On the current source, direct host tests pass (`framework-location` 7/7; `ios-location` unit tests 33/33; integration tests 2/2), and the Python source/negative adapter passes. These results do not replace the blocked registered profile. No live Core Location delivery, permission prompt/restoration, Simulator execution, or physical-device behavior is claimed.

## Additional reproduction — 2026-10-12 (B570)

The same suspected defect blocks B570's registered Bluetooth profiles. On clean committed framework revision `faa8fe972f123cd5ca3f8979bad28a38629208f`, both of these commands exited 1 before any validation gate ran:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability framework-bluetooth --format json
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-bluetooth --format json
```

Both returned `{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}`. `--explain ios-bluetooth` succeeds, and its schema-v1 spec parses. The selected binary reports source SHA `350971094cc3b77191cd6b3d07e01af04d71943d`, release `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`, and SHA-256 `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`. Host is `x86_64-apple-darwin`, macOS 26.6.2, Xcode 26.6 / iPhoneOS SDK 26.5, and Rust 1.94.1. No validator engine source or pinned tooling artifact was inspected or changed.

The B570 isolated workstream's prior target compilation, Clippy, Rustdoc, and link-fixture results remain limited to its own source snapshot; they do not qualify the integrated `ios-bluetooth` profile. The current registered profile is unverified in the root checkout, and no radio, Simulator, or physical-device behavior is claimed.
