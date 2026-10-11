# Tooling bug: validator rejects a stable source snapshot

- State: suspected tooling defect
- Affected tool: `ios-rust-validate`
- Tool version, release ID, full source SHA: `0.1.0`, `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`, `350971094cc3b77191cd6b3d07e01af04d71943d`
- Binary SHA-256: `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`, macOS `26.6.2`, Xcode `26.6` build `17F113`, iOS SDK `26.5`, `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: `ios-connection`, `ios-files`, `ios-runtime`; focused registered API validation

## Minimal reproduction

From a clean committed `main` worktree with the valid repository spec at 258,530 bytes:

```sh
python3 -m json.tool tools/validation/specs/validation-v1.json >/dev/null
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --list
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-connection
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-connection
```

The first two commands succeed. The capability command exits before reporting any gates. Repeating
with `ios-files` and `ios-runtime` produces the same result. A temporary valid one-profile
`ios-connection` spec under the repository also passes `--list` and `--explain`, then fails on
`--capability` with the same diagnostic; the temporary spec was removed after reproduction

## Expected behavior

The installed validator documents local source snapshots, including relevant worktree inputs, in
`docs/SHARED_TOOLING.md`. For a valid, stable source tree and profile, `--capability ID` should
capture the source and proceed to the declared gates

## Actual behavior

The command exits 1 and returns:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

The same message occurs before any capability gates for the listed profiles. No PASS result is
available for these final registered profile runs

## Troubleshooting already performed

- Verified `command -v`, full `--version --format json` release/source metadata, and binary SHA-256
- Confirmed the host, Xcode/SDK, and Rust toolchain versions above
- Validated the full spec with `python3 -m json.tool`; `--list` and `--explain` succeed; spec size is below the 262,144-byte limit
- Reproduced with both the full spec and a valid temporary one-profile spec under the workspace root
- Checked `git status`; the worktree was clean at the committed reproduction point
- Hashed every tracked and nonignored untracked file before and after a failing attempt; no file content changed
- Confirmed no Cargo or validator process was active outside the reproduction command
- Confirmed the final tree still passes the focused iOS Files release link/import script, documentation index check, zero-Swift-source check, and generated-report freshness check; these do not replace the blocked validator gates

## Impact

The final `ios-connection`, `ios-files`, and `ios-runtime` registered checks cannot be reported as
passed from this source snapshot. Existing agent-reported B532 host tests and device/Simulator
compile/link checks, and the direct iOS Files link/import result, remain separate evidence. No
required validator check was bypassed or weakened. Independent implementation work can continue

## Recommended maintenance investigation

Inspect the installed release's source-snapshot consistency check and its set of files/metadata
treated as mutable during capture. No cause or fix is asserted from this black-box reproduction
