# PLAN_VALIDATION.md — Active validation obligations and evidence

## Authority and scope

This is the active cross-cutting G-family validation contract. Original G1–G131 evidence and accumulated decisions remain recoverable in the historical `PLAN_VALIDATION.md` at Git `36e3090a79a2fdbc8f348ccb89fe773c3dbb1247`. Completed or consolidated bounded gate IDs remain permanent; do not reactivate them solely because their evidence is included here. Reverify the current `main` SHA and consult each assigned `PLAN_VALIDATION_*.md` only where its distinct acceptance criteria remain open.

## Current tool architecture

`ios-rust-validate` and `ios-rust-build` are independently installed, SHA-256-verified PATH executables. Read `docs/SHARED_TOOLING.md` for installation, schemas, `--help`, `--version`, `--list`, `--explain`, `--capability`, `--changed` and `--all`. The repo-local specs are `tools/validation/specs/validation-v1.json`, `tools/validation/specs/schema-v1.json`, and the per-capability `build-spec.json` where applicable. Rust engine source no longer lives in ordinary `main`. Non-validation `xtask` functionality remains available for `docs-check`, `zero-swift-source`, no-std, ABI, codegen, dependency, SDK and link audits.

Only four production validation pilots are currently registered: `ios-homekit-identify-status`, `ios-activitykit-status`, `ios-alarmkit-status`, and `ios-photogrammetry-status`. Existing checks for other capabilities remain authoritative until individually migrated. **Never call an unregistered capability validated or delete its legacy gates based on tool availability alone.**

## Required implementation / test invariants

For every new or modified capability:

1. Inspect exact API semantics and original negative assertions; map the changed files and reverse dependencies. Add declarative checks to the installed validator only if its schema accurately expresses every required assertion. Use a focused Python 3 adapter for specialized file/fixture checks and genuine native test binaries/compiler oracles for ABI details.
2. Preserve Cargo format, locked compilation, strict Clippy, rustdoc, target-specific device and simulator checks, feature isolation, framework and native symbol import/export scans, linker deployment floors, API availability, weak linking, docs and zero-shipping-Swift policies as applicable. Exact compiler-derived calling conventions and object ownership are not substitutable with string searches.
3. Gate the C ABI through independent C11 and C++17 consumers, stable headers/symbols/versioning, callback/error conventions, pointer/ownership semantics and no unwind through FFI. Validate async request/completion/cancellation as separate states, not generic success/failure.
4. Keep portable `no_std` checks and minimal linkable binaries, dependency feature-tree audits, and size/performance evidence for proposed replacements. Do not claim a performance improvement without comparable measured baseline.
5. Track static compile/link/import evidence separately from simulator execution, device execution, signing/entitlement conditions, permission prompts, background behavior, assistive interaction and other hardware-dependent behavior. Missing SDK, target or hardware is `SKIPPED`/blocked with explicit reason, never `PASS`.
6. For migrations, capture the baseline legacy test inventory, add positive and deliberately failing fixtures for each negative rule, prove equivalence of actual commands and expected outcomes on supported hosts, and only then retire duplicated wrappers. Command snapshots are useful evidence, not sufficient alone.
7. For new source paths, configure `--changed EXPLICIT_BASE` path rules and reverse dependency edges so shared ABI/helper changes select all impacted consumers. Include tracked/untracked/deleted-file cases in deterministic tests; fall back to a safe superset for unknown impacts.

## Known open evidence and limits

- Prior Apple static proofs used Xcode 26.6 and iOS SDK 26.5; planned Xcode 27.x qualification remains open until explicitly established. On-host compilation does not establish Apple runtime/device behavior.
- `xtask parity` was previously unavailable without an Apple reference-vs-Rust candidate suite; do not claim Apple parity. Likewise, assembly/codegen audit is not a substitute for workload benchmarks.
- The historical Ubuntu non-Apple `objc2` Clippy failure remains open and requires explicit triage; do not silently disable gates or claim whole-CI success.
- The historical macOS G111 iOS Files import-order failure did not reproduce at `8934b3992c00b5ea70c6aa6e0f7713d7dc405281`. `PATH="$PWD/target/ios-rust-tools/bin:$PATH" sh platform/ios/ios-files/check-app-data-link-imports.sh` passed with pinned `ios-rust-build`/`ios-rust-validate` 0.1.0 tools installed, Cargo/Rust 1.94.1, Xcode 26.6 and iOS SDK 26.5. Device and Simulator Release artifacts both matched the exact sorted import allowlists: `ios-files` imported CoreFoundation, Foundation, `libSystem.B.dylib`, `libiconv.2.dylib` and `libobjc.A.dylib`; `ios-preferences` imported Foundation, `libSystem.B.dylib` and `libobjc.A.dylib`. Both device artifacts reported minos 10.0 and both Simulator artifacts minos 14.0. The focused gate also passed its symbol/selector and no-Swift assertions. This is current static gate evidence only: it does not establish a rerun of the historical CI job, Xcode 27.x qualification or runtime/device behavior.
- Completion evidence for prior bounded gate groups (including consolidated G8, G11, G14–G16 and G60) remains historical. Their unresolved downstream runtime evidence stays assigned to the owning capability/validation workstream; a closed compiler-gate workstream does not close an entire capability.

## Ownership, commands and handoff

The **capability owner** owns its package-local tests, fixtures, declarative spec entries and API-specific Python adapters; the **G/integration owner** owns CI coverage and cross-workstream validation consistency. The shared Rust engine is separately maintained; ordinary API Codex never edits/retrieves its source. If documented CLI behavior is defective, sanitize reproducible evidence in `BUG_REPORT_<NAME>.md` and stop affected work.

Minimum orientation from the repo root after installing tools:

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --list
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-activitykit-status
cargo +1.94.1 xtask docs-check
cargo +1.94.1 xtask zero-swift-source
```

Run `--capability ID` only if registered; run appropriate existing `platform/ios/**/check.sh` and legacy ABI gates otherwise. Report command/executor environment, exact results, skipped gates, target/deployment floors, tool versions, negative-test evidence, commit SHA, and remaining physical-device obligations. Review final diff for gate removal, accidental source import, missing reverse-dependency selection and false completion claims.

## Completed-tooling migration boundary

The validator executable is already implemented and pinned. R1/R2 implementation plans were removed in `f3b14892a560da05a086caf9e6d30a0c9beda25e`. Do not reopen them to register new API coverage. Before retiring an existing focused check, document its full positive/negative gate inventory and demonstrate supported-host parity using the actual installed version; otherwise retain the focused check alongside any new declaration. Tooling bugs belong to a separate maintenance session via the documented `BUG_REPORT_*.md` workflow.