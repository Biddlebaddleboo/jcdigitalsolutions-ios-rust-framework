# PLAN_VALIDATION_IOS_SAFARI.md — G58: SafariServices Compile and Link Gates

## Status and objective

B64/G58's static build and link/import checks for the typed SafariServices wrapper were integrated and previously passed. This is not a browser runtime or controller presentation test. Preserve required evidence; use the completed PATH tools for standardized future gate execution. First inspect `platform/ios/ios-safari/Cargo.toml`, `platform/ios/ios-safari/check-link-imports.sh`, the B64 wrapper, and `docs/SHARED_TOOLING.md`.

## Required existing gates

- Locked `ios-safari` target `cargo check` on `aarch64-apple-ios` and `aarch64-apple-ios-sim`, and strict `cargo clippy --all-targets -- -D warnings` on both targets.
- Formatting of the capability crate, crate rustdoc, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`.
- `sh platform/ios/ios-safari/check-link-imports.sh` must build arm64 device and simulator probes, check the direct SafariServices import allowlist, expected class/selector strings, and Objective-C lookup/message imports, and reject forbidden/unrelated imports.
- Verify device minimum iOS **10.0** and arm64 simulator minimum iOS **14.0** with `vtool -show-build` as in the existing probe. SafariServices initializer availability in the SDK begins with iOS 9.0, but this Rust device target imposes the stricter iOS 10.0 build floor. Do not conflate the two.
- Historical passing evidence includes root workspace check, all-target/all-feature Clippy after the link-probe example's iOS guard and host no-op main, and inspected but **not executed** probe binaries. Revalidate any changed gate under the actual current host; hosted Xcode 27 static-gate evidence is recorded below, but runtime behavior remains unverified.

## Shared validator integration

The current four-pilot `tools/validation/specs/validation-v1.json` does **not** register `ios-safari`. Do not claim `ios-rust-validate --capability ios-safari` passes. For additional changes, express routine checks in schema-v1 only when the new profile reproduces every compiler/import/OS-floor/negative gate with demonstrated positive and negative parity. Keep the focused script in place otherwise. A Python adapter is permitted solely for bounded Safari-specific deterministic checks, not a substitute for linked artifact inspection or live UIKit/Safari rendering. Tool-engine defects belong in a separately escalated sanitized `BUG_REPORT_*.md`.

## Non-goals, validation and handoff

Do not claim live controller creation, UI presentation, navigation, URL request, page load, Safari routing, privacy prompt, network response or browser parity. No runtime tests were executed by G58. Report full commands/target toolchain, direct imports and OS-floor proof, skipped tests, changed files and SHA; preserve all original public-API constraints.

## Hosted Xcode 27 static-gate evidence (2026-10-10)

Run: [GitHub Actions run 38075483431](https://github.com/Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework/actions/runs/38075483431); Xcode 27 lane; source SHA `85db105389c1d0b212bc385d9b4b6a1f6e049c0b`

Pass steps: `Check iOS SafariServices device target`, `Check iOS SafariServices simulator target`, `Clippy iOS SafariServices device target`, `Clippy iOS SafariServices simulator target`, `Audit iOS SafariServices link imports`

Link cmd: `sh platform/ios/ios-safari/check-link-imports.sh`; device/Simulator Release probe link; SafariServices import allowlist, symbol and OS-floor checks

Other pass gates: `cargo fmt --all -- --check`, `cargo doc --locked --workspace --no-deps` with `ios-safari`, `cargo xtask docs-check`; docs-index and zero-Swift-source check in docs-check log

Toolchain: Xcode 27.0 build 27A266a; iPhoneOS and iPhoneSimulator SDK 27.0; Rust 1.94.1

Gate paths: `platform/ios/ios-safari/`, `crates/framework-format/`, `platform/ios/ios-runtime/`, `xtask/`, and workspace manifest/lockfile; no diff from run SHA to `origin/main` `7bd07d8`. Workflow delta adds only Core NFC link/import audit; G58 step definitions unchanged

Gap: no `git diff --check` cmd in hosted log; required gate unverified; full G58 gate set not proven by this run

Limit: static link/import and OS-floor proof only; no probe run; no Safari controller, navigation, URL, page-load, privacy prompt, network response, or browser parity proof

The hosted log has no `git diff --check` cmd, so full G58 gate coverage remains unproven. Static link/import and OS-floor proof only; no probe run. No Safari controller presentation, navigation, URL request, page load, privacy prompt, network response, or browser parity claim
