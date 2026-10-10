# PLAN_VALIDATION_ARCHIVE.md — Workstream G2: Xcode Archive Smoke

## Status

## Existing shared tooling boundary

R1/R2 installed PATH tools are completed and do not replace this G2 **real Xcode archive** objective. Keep `cargo xtask archive-smoke` and its unsigned archive, plist, Mach-O, linkage and signing checks; do not infer a successful archive from `ios-rust-validate` package-level PASS, cross-target `cargo check`, or source-pattern assertions. If registering archive-related checks in a declarative validation profile, preserve the real archive command, artifact inspection and the distinction between unsigned packaging, signed export and launch on simulator/device. The existing archive evidence below is historical and target/toolchain-scoped; missing signing or physical-device evidence remains open. See `docs/SHARED_TOOLING.md` for installed tool usage.


- Result: `cargo xtask archive-smoke` passed on 2026-10-07 and twice on 2026-10-08 with Xcode 26.6 (build 17F113) and iPhoneOS SDK 26.5; the latest run follows the adaptive-color and callback-smoke app changes
- Evidence: Cargo Release device app; shared `ios-minimal` Xcode scheme; archive/app plist lint; app executable import scan; no Swift/Python runtime import, no `.swift` file in archive, and no app code signature, `_CodeSignature`, or `embedded.mobileprovision`
- Archive path: `target/ios-minimal/archive/ios-minimal.xcarchive`
- Scope: unsigned archive evidence only; Xcode 26.6 and SDK 26.5 fall below Xcode 27.x plan baseline
- CI: manual macOS archive check; CI has simulator Release app build and import/plist check only, no archive action
- Supplemental runtime: the x86_64 iOS 18.0 simulator app launched on 2026-10-08 and displayed the label/button in light and dark appearance. With `--exercise-rust-button-callback`, `UIControl::sendActionsForControlEvents(UIControlEvents::TouchUpInside)` invoked the Rust callback and changed the label; this was programmatic UIKit dispatch, not a user touch
- Open: archive code sign, provisioning, archive export, arm64 simulator launch/install, and physical-device run

## Objective

Turn the explicit `archive-smoke` unavailable result into a real public-Xcode archive check for the Rust-owned minimal iOS example. The result must not require repository-authored Swift or a signing certificate for the unsigned validation path.

## Dependencies

- Workstream B minimal example and `cargo xtask ios-build --device --release` are integrated
- Workstream G host tooling is integrated
- Xcode and iPhoneOS SDK are available on the host

## Write scope

- Xcode archive project/scheme and scripts scoped to `examples/ios-minimal/**`
- archive command integration and focused checks under `tools/xtask/**`
- `docs/IOS_BUILD.md` and `docs/VALIDATION.md`

The archive flow may consume the existing Rust-built app bundle. Do not add Swift, change the UIKit runtime semantics, claim signing/provisioning, or modify portable APIs. Preserve exact Xcode/SDK versions and distinguish a successful unsigned archive from a signed App Store archive.

## Required behavior

- `cargo xtask archive-smoke` invokes a reproducible public `xcodebuild archive` path for the minimal app on a supported host.
- The archive path builds the Release device executable with Cargo, packages the app with the existing Info.plist, and uses Xcode's standard archive action.
- A host without Xcode/SDK or without a supported scheme fails before claiming archive success and reports the exact missing input or command error.
- The archive's app contains the expected executable and Info.plist; Mach-O imports contain no Swift or Python runtime; the archive contains no generated or checked-in `.swift` source.
- CI or a documented manual command runs this check only on macOS with the necessary SDK. Certificate-dependent signing/export remains a separate gate.

## Validation and handoff

Run `cargo xtask archive-smoke` and inspect the produced `.xcarchive` with Xcode tooling. Report the exact command, Xcode version/build, output path, signing state, Mach-O imports, plist result, and any host-only limits. Do not convert an unavailable or unsigned result into a claim of signed archive support.
