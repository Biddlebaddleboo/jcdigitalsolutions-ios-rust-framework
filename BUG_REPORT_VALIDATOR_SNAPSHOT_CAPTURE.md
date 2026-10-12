# Tooling bug: clean capability snapshot capture is unstable

- State: suspected tooling defect
- Affected tool: `ios-rust-validate`
- Tool version, release ID, full source SHA: `0.1.0`, `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`, `350971094cc3b77191cd6b3d07e01af04d71943d`
- Executable SHA-256: `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`; SDK gates were not reached; `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: `ios-bluetooth`, B598 connected-peer GATT notifications

## Minimal reproduction

On clean commit `10c1d486596c4a1b9c0cc4a61d62c4824ff56157`, with `git status --short` empty:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-bluetooth
```

The profile and spec resolve through `--list` and `--explain ios-bluetooth`. The spec is 262,086 bytes, below the tool's 262,144-byte limit.

## Expected behavior

The selected capability is snapshotted and its registered gates run. The repository workflow documents immutable source snapshots and requires `--capability ID` for focused API validation (`AGENTS.md`, “Required validation registration for API work”; `docs/SHARED_TOOLING.md`, sections 4 and 7).

## Actual behavior

Two runs on that clean commit, one with default output and one with `--format json`, exited 1 before a snapshot ID or any gate result and returned:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

No source path or conflicting content is identified. These runs are not validation passes.

## Troubleshooting already performed

- Verified executable version, release ID, source SHA, host, and SHA-256.
- Verified `--list` and `--explain ios-bluetooth` with the repository spec.
- Verified the workspace was clean at the reported commit and the spec stayed below its byte limit.
- Repeated the exact capability command twice on that stable clean commit.
- A prior snapshot on commit `c954248` reached 34 checks: 29 PASS, 3 Clippy FAIL, 2 SKIPPED. The Clippy issue was fixed before commit `10c1d48`; direct host Clippy and device/Simulator compilation then passed. That earlier snapshot does not validate the final commit.

## Impact

The final registered `ios-bluetooth` run is blocked at source capture, so its required checks are unverified for B598. Independent Cargo host tests, host Clippy, and device/Simulator target compiles can run without this snapshot operation; they do not replace the registered validator result.

## Evidence and attachments

The safe structured diagnostic above is the complete output from both final attempts. No credentials, account data, or private environment values are included.

## Recommended maintenance investigation

Investigate why a clean, committed workspace fails the source-consistency check before any capability gate, and add a diagnostic that identifies the changed input when capture is unstable.
