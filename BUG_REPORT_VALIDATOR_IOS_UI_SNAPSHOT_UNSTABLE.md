# Tooling bug: stable workspace reports snapshot-unstable

- State: suspected tooling defect
- Affected tool: `ios-rust-validate`
- Tool version, release ID, full source SHA: `0.1.0`, `ios-rust-tools-0.1.0+350971094cc3b77191cd6b3d07e01af04d71943d`, `350971094cc3b77191cd6b3d07e01af04d71943d`
- Binary SHA-256: `77f9c7aa941ba1335827ffc6ce274898987ce1d5f408d91d1a7c1daf34527944`
- Host triple, OS, SDK, Rust toolchain: `x86_64-apple-darwin`, macOS `26.6.2`, Xcode `26.6` (build `17F113`), iOS SDK `26.5`, `rustc 1.94.1 (e408947bf 2026-03-25)`
- Affected capability and task: `ios-ui` / B538 selectable text registration; independent reproduction on `framework-ui`

## Minimal reproduction

From the clean committed framework root at `1c11d629aee6d1a22b111f9eee4888e4d4753864`, with the checked-in spec:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability ios-ui
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --capability framework-ui
```

Both commands exit `1` before any gate runs. `ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-ui` succeeds and lists the profile.

## Expected behavior

The documented capability command captures a stable workspace snapshot and runs its registered gates. `docs/SHARED_TOOLING.md` describes source snapshots and immutable snapshot results; a clean committed source tree should not be rejected as changing during capture.

## Actual behavior

Both minimal-reproduction commands return:

```json
{"code":"validation_error","message":"error-blocked: snapshot-unstable: source changed during capture","schema_version":1,"status":"error-blocked"}
```

Earlier, the `ios-ui` profile did capture and execute 273 checks on snapshot `fe303d166778861adc52443af2abcad1aff50eaebe9f3675435867dbd43be560`: 269 passed, two stale source guards failed, and two runtime checks were skipped. The stale guards were corrected and the spec was re-explained successfully. Subsequent `ios-ui` attempts, including from the clean committed tree, and the independent `framework-ui` reproduction failed at snapshot capture. An explicit-base `--changed 0a54b83c6651876ea8e939e32981a2b438363067` attempt also returned the same error before the root integration commit.

## Troubleshooting already performed

- Confirmed executable path `/Users/john/.local/bin/ios-rust-validate`, full version metadata, source SHA, and binary checksum above.
- Verified the updated JSON parses and `--explain ios-ui` recognizes the two B538 guards, the existing ABI fixtures, iOS 12.0 / 14.0 targets, and profile gates.
- Confirmed clean root `git status`, stable `HEAD` and `origin/main`, no active validator/Cargo/rustc processes, and no recent changes under `platform/ios/ios-ui` or `crates/framework-ui` before the clean-tree reproductions.
- Repeated the documented capability command and tested an unrelated registered capability; both fail identically before executing gates.
- No engine source, pinned archive, release manifest, or required gate was changed or bypassed.

## Impact and safe continuation

The integrated root `ios-ui` profile remains unverified after B538, and a clean `framework-ui` profile cannot be captured. B538's package-only host tests, Device/Simulator compilation, and release link/import/deployment probes passed on its isolated workstream before integration; those results do not replace the registered profile or prove runtime selection. Continue only independent API work and report any profile outcomes truthfully until a separately authorized tooling-maintenance session diagnoses this suspected defect.
