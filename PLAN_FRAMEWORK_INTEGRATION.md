# PLAN_FRAMEWORK_INTEGRATION.md — Framework audits and verified tooling promotion

**Repo:** `jcdigitalsolutions/jcdigitalsolutions-ios-rust-framework`; baseline `2f32533c787b5753725a1f7eb876bbdd38908547`. This is the consumer workstream; dependent on tooling workstreams A+B+C. Read PLAN_TOOLING_UPGRADE.md.

## Implementation scope
Start at `tools/releases/manifest-v1.tsv`, `tools/install-tools.sh`, `.github/workflows/shared-tools.yml`, `.github/workflows/ci.yml`, `docs/SHARED_TOOLING.md`, `docs/VALIDATION.md`, `tools/native-build/specs/schema-v1.json`, `tools/validation/specs/validation-v1.json`, `docs/capabilities/capability-status.json`, `tools/xtask/src/sdk_inventory.rs`, `tools/xtask/src/main.rs`, `reports.yaml`, bug reports `BUG_REPORT_{IOS_RUST_BUILD_NO_FRAMEWORK,VALIDATOR_SNAPSHOT_UNSTABLE,VALIDATOR_SOURCE_SNAPSHOT_UNSTABLE,VALIDATOR_IOS_UI_SNAPSHOT_UNSTABLE}.md`. Before editing check current versions and overlap with active plans; don't overwrite unrelated root `PLAN.md`.

## Required changes
- Sync the build-schema contract with tooling's empty-framework support. Verify the existing system logging C build spec, and register it only if build.rs/source are present and support the target; never fabricate a framework dependency.
- Add a bounded deterministic **capability consistency audit** through `cargo xtask` or equivalent established command: cross-check Cargo metadata, supported-capability crate names, validation IDs and dependency edges, manifest paths, required native build specs and generated report references; allow explicit capability exclusions and cross-platform abstract contracts. Fail on duplicate IDs, dangling refs, malformed OS floor, unsupported target and stale generated output. New tests should use minimal fixture manifests/registries rather than rewriting huge source JSON.
- Improve `sdk-inventory` only with stable metadata/limited public declaration mapping and explicit limitations; avoid claiming API-level completeness from header line counts. Fingerprint SDK/toolchain, stable framework/path ordering; no private headers or undocumented Apple APIs.
- Register high-value deterministic behavior/ABI fixtures for location, Bluetooth, notifications, background lease, FFI callback life cycle and UIKit where the current frameworks expose testable boundaries. Prefer fake clocks and callback event streams; don't replace real native parity with mocks or force all profiles to use Python adapters. Keep existing schema/safety gates intact.
- Once newer generic tooling passes A+B+C, build and qualify macOS arm64/x86_64 and Linux x86_64 binaries. Release with new **distinct** semver/release ID/full source SHA/host triple/checksum and documented supported schema/command features. Promote archives and manifest together; installer verifies checksum and binary metadata. Preserve last-known-good rollback and manual-only Actions triggers. Confirm `--version --format json`, `--help`, `--list`, `--explain`, `--capability` and `generate-reports --check` on promoted binaries.
- Update docs and bug reports with exact new tool fingerprints, repro results, root cause only if established, explicit current gate results and unresolved live-platform limitations. Do not mark a suspect bug closed solely because an isolated worktree happens to succeed.

## Deterministic coverage requirements
- Fixture intentionally removes package/validation mapping, duplicates ID, changes Cargo feature/edge, uses invalid path, changes schema or stale generated page -> exact failure.
- Same pinned SDK and source -> byte-equivalent inventory/evidence except separately identified environment fields; SDK bump -> changed fingerprint.
- Release manifests reject corrupt archive, mismatched source SHA, wrong host triple and old-semver collision; rollback after interrupted promotion.
- Existing framework host cargo unit checks, target checks and native link smoke remain. Simulator/device runtime proof requires real environment; never infer from `cargo check`.

## Validation sequence
`cargo +1.94.1 fmt --all -- --check`; relevant `cargo +1.94.1 test --locked ...`; `cargo xtask docs-check`; `cargo xtask sdk-inventory --sdk all --output target/xtask/sdk-inventory-all.json`; `cargo xtask dependency-audit`; run pinned installer and validator version/known profiles; run focused native logging build fixture. Manual Actions qualification on release SHA after approval. Do not promote if snapshot errors remain unexplained/uncaught.

## Handoff
Report framework SHA, release manifest/artifact SHAs, changed files, every gate result including skips, retained rollback, and known SDK/device gaps. Keep existing product code untouched; remove temporary plan artifacts only at end and only those introduced by this execution.