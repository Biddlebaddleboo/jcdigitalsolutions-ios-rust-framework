# Validation note: primary-worktree snapshot instability

- State: worktree-specific instability observed; no confirmed tooling defect
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

The first two commands succeed. In the primary worktree, the capability command exits before
reporting gates. The same error repeated for `ios-files` and `ios-runtime`, including after the
implementation and report commits were pushed and the worktree was clean. A temporary valid
one-profile `ios-connection` spec under the repository also passed `--list` and `--explain`, then
failed on `--capability`; the temporary spec was removed after reproduction

## Expected behavior

The installed validator documents local source snapshots, including relevant worktree inputs, in
`docs/SHARED_TOOLING.md`. For a valid, stable source tree and profile, `--capability ID` should
capture the source and proceed to the declared gates

## Actual behavior

The command exits 1 and returns:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

The same message occurs before any capability gates in the primary worktree. Detached clean
worktrees at the successive pushed `main` commits captured successfully and ran the affected
profiles: `ios-connection` passed 53 checks with 2 runtime skips and `ios-files` passed 50 with 2
runtime skips at `37efeb5`; `ios-runtime` passed 43 with 2 runtime skips at `3470d7f`; and `ios-ui`
passed 253 with 2 runtime skips at `f298fd1`, after its Rustfmt mismatch was fixed. These isolated
worktree runs establish the profile results; the primary-worktree capture failure remains unexplained

## Troubleshooting already performed

- Verified `command -v`, full `--version --format json` release/source metadata, and binary SHA-256
- Confirmed the host, Xcode/SDK, and Rust toolchain versions above
- Validated the full spec with `python3 -m json.tool`; `--list` and `--explain` succeed; spec size is below the 262,144-byte limit
- Reproduced with both the full spec and a valid temporary one-profile spec under the workspace root
- Checked `git status`; the worktree was clean at the committed reproduction point
- Hashed every tracked and nonignored untracked file before and after a failing primary-worktree attempt; no file content changed; this did not cover ignored/generated files or concurrent processes in other worktrees
- Confirmed the final tree passes focused profiles from a detached clean worktree at the same pushed `main` commit; no product-source changes were needed to make snapshot capture succeed there
- The original workspace contains other worktrees and ignored build output; their activity/content was not isolated during the failing captures

## Impact

The primary worktree cannot currently be used for validator source capture. The detached clean
worktree ran the required profiles successfully, with simulator execution and physical-device
behavior still skipped. No required validator check was bypassed or weakened

## Recommended maintenance investigation

If the primary-worktree issue recurs, isolate ignored/generated files and concurrent worktree activity
before treating it as an engine defect. No engine maintenance change or cause is asserted from this
black-box evidence
