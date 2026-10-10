# PLAN_VALIDATION_IOS_NETWORK.md — G4: iOS Network Target Gates

## Status and scope

G4's macOS static `ios-network` gates and documentation were integrated. This plan remains authoritative for preserving their correctness and for explicitly outstanding URLSession runtime/parity evidence, **not** for rebuilding R1/R2 engines. Start with `PLAN_IOS_NETWORK.md`, `.github/workflows/ci.yml`, `platform/ios/ios-network/check-link-imports.sh`, `docs/ios/network.md`, and `docs/VALIDATION.md`.

## Verified facts

- Historically, locked checks and strict all-target Clippy passed for `ios-network` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`; CI includes those checks after target installation.
- The package link probe verified `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib` direct imports, rejecting specified Swift/Python runtime and unrelated capability patterns. The probe *does not run* or start a URLSession request.
- Ten deterministic host unit tests passed for conversions and completion/drop races. The link-only example explicitly drops its unpolled future without starting a request.
- CI YAML parsing, `cargo xtask docs-check`, and `git diff --check` previously passed. This is historical evidence only; subsequent changes require revalidation.
- No Apple runtime reference implementation/result, deterministic local-server differential, real URLSession execution, or measured HTTP replacement parity exists. Xcode 26.6/SDK 26.5 is not Xcode 27.x qualification.

## Installed tooling usage and requirements

`ios-rust-build` and `ios-rust-validate` are installed, source-isolated tools (`docs/SHARED_TOOLING.md`). `ios-network` is **not** a registered four-pilot validator capability in the inspected `tools/validation/specs/validation-v1.json`; do not invent a passing `--capability ios-network` invocation. Keep the following gates in CI/local execution unless a schema-v1 profile is added and exact positive/negative parity established:

- Locked device and simulator `cargo check`; locked strict `cargo clippy --all-targets -- -D warnings` on both Apple targets.
- `sh platform/ios/ios-network/check-link-imports.sh`, retaining exact import allowlist, excluded dependencies and compile/link-only proof.
- Existing host unit tests, CI YAML syntax verification, docs-check and diff-check.
- Preserve non-network-dependent CI: no public endpoint requests, timing sleeps or external HTTP service dependencies.

A new profile may factor routine Cargo checks into declarative gates; specialized binary imports, async cancellation/drop races and runtime tests cannot be replaced by Python text-pattern assertions. Engine fixes are not G4 work; create `BUG_REPORT_*.md` for a reproducible tool defect and stop that affected work.

## Open parity and performance requirements

Do not claim runtime parity without a named candidate and an Apple runner exercising current URLSession and the candidate against **the same deterministic local fixture**, with normalized response/error comparison. Record runner/app revision, Xcode/SDK, device/simulator model and OS build, fixture revision, request, status/headers/body, normalized error results, cancellation/drop behavior, and deviations. Benchmarking additionally requires predeclared representative physical-device Release A/B evidence under `PLAN_REPLACEMENTS_HTTP.md` (E1); scaffolded harnesses are not measurements. Keep parity, elapsed time and linked-artifact evidence distinct.

## Ownership and handoff

G4 owns only matching scoped validation/CI documentation; product HTTP semantics, iOS backend implementation and global manifest remain with their existing workstreams. Verify current `main`; list changed files, exact checks (including skipped/unavailable gates), CI YAML result, SHA, and unresolved reference/runtime evidence. Avoid broad source searches unless paths moved or failing code requires them.