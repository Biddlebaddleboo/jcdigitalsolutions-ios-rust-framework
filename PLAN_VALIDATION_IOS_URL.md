# PLAN_VALIDATION_IOS_URL.md — Workstream G14: iOS URL Gates

## Status

CI wiring, the B20 `ios-url` package, and its import script are present. On 2026-10-08, device /
Simulator checks, strict all-target Clippy, and the corrected device/Simulator import audit passed
locally on Rust 1.94.1 with Xcode 26.6 build 17F113 and SDK 26.5. Both arm64 probes import exactly
Foundation.framework, `/usr/lib/libobjc.A.dylib`, and `/usr/lib/libSystem.B.dylib`; `strings`
contains `NSString`, `NSURL`, and `URLWithString:encodingInvalidCharacters:` but not the legacy
`URLWithString:` selector. `vtool` reports iOS 17.0 minimum on both targets. Probes were linked,
not executed; no passing CI workflow run or native parser / URL-open behavior is claimed

## Objective

Add macOS CI and local evidence for the D8 / B20 strict Foundation URL value adapter

## Dependencies

- D8 `framework-format::Uri` is integrated
- B20 `ios-url` is integrated; see `PLAN_IOS_URL.md`
- G1 validation tooling and CI are integrated

## Write scope

- `PLAN_VALIDATION_IOS_URL.md`
- `.github/workflows/ci.yml`
- concise G14 evidence in `PLAN_VALIDATION.md` and `docs/VALIDATION.md`

Do not edit B20 APIs or platform source, Cargo manifests/lockfile, the shared capability matrix,
or other target gates. Root owns capability counts and shared indexes

## Required gates

- On macOS, install `aarch64-apple-ios` and `aarch64-apple-ios-sim`
- Run locked device and Simulator `cargo check -p ios-url` and strict all-target Clippy with
  `-D warnings` for both targets
- Run `sh platform/ios/ios-url/check-link-imports.sh`; verify exact Foundation/system imports and
  required Objective-C runtime imports while rejecting Swift/Python runtime and unrelated
  frameworks. Its `strings` audit must contain `URLWithString:encodingInvalidCharacters:` and
  omit the legacy `URLWithString:` selector
- The script must build, not run, link probes with iOS 17.0 deployment metadata for device and
  Simulator, and confirm the strict NSURL selector string is present
- Retain host formatting, workspace Clippy, `cargo xtask docs-check`, and
  `cargo xtask zero-swift-source` gates
- Keep runtime parsing, URL opening, network, and performance behavior out of CI claims

## Validation and handoff

- Root reconciled Cargo.lock/workspace before the target checks. These exact commands passed on
  Rust 1.94.1: `cargo +1.94.1 check --locked -p ios-url --target aarch64-apple-ios`,
  `cargo +1.94.1 check --locked -p ios-url --target aarch64-apple-ios-sim`,
  `cargo +1.94.1 clippy --locked -p ios-url --all-targets --target aarch64-apple-ios -- -D warnings`,
  `cargo +1.94.1 clippy --locked -p ios-url --all-targets --target aarch64-apple-ios-sim -- -D warnings`,
  and `sh platform/ios/ios-url/check-link-imports.sh` after the script gate correction
- The first `sh platform/ios/ios-url/check-link-imports.sh` attempt failed because it expected
  `_OBJC_CLASS_$_NSString`, which was absent. The gate was corrected to require Objective-C
  runtime symbols plus Foundation class and selector strings; the full script then passed for
  device and Simulator
- The script-built arm64 artifacts are
  `target/ios-url-link-aarch64-apple-ios/aarch64-apple-ios/release/examples/ios_url_link_import_probe`
  and
  `target/ios-url-link-aarch64-apple-ios-sim/aarch64-apple-ios-sim/release/examples/ios_url_link_import_probe`.
  `otool -L` reports exactly Foundation.framework, `/usr/lib/libobjc.A.dylib`, and
  `/usr/lib/libSystem.B.dylib` for both. `nm -u` confirms `_objc_alloc`, `_objc_getClass`, and
  `_objc_msgSend` plus retain/release and selector symbols, with no forbidden runtime or
  unrelated-capability symbols. `strings` contains `NSString`, `NSURL`, and
  `URLWithString:encodingInvalidCharacters:`; the legacy `URLWithString:` selector is absent
- `vtool -show-build` reports `IOS` / `IOSSIMULATOR`, minimum iOS 17.0 for both, and SDK 26.5.
  The B20 API floor is iOS 17.0; the host SDK is 26.5. Xcode 26.6 build 17F113 is below the
  planned Xcode 27.x baseline
- The device and Simulator probes were linked, not executed. These checks do not prove `NSURL`
  accepts every D8 `Uri`, component parity, runtime availability in an app with an incorrect
  deployment target, URL opening, or performance. No passing CI workflow run is recorded
- Run `cargo xtask docs-check`, `cargo xtask zero-swift-source`, workflow YAML parsing, and
  `git diff --check`; do not edit shared capability totals
- State that link/import evidence does not prove `NSURL` accepts every D8 `Uri`, component parity,
  runtime availability in an app with an incorrect deployment target, URL opening, or performance
