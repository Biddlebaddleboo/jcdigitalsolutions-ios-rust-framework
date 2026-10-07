# PLAN_VALIDATION_ARCHIVE.md — Workstream G2: Xcode Archive Smoke

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
