# iOS Build and Packaging Direction

## Current build status

The architecture plan names Xcode 27.x as its execution-host baseline. The recorded host has Xcode 26.6 (build 17F113) and SDK 26.5, so it is below that baseline. The repository has a Cargo-based minimal iOS example, but no full framework Xcode project/workspace or iOS framework crate. On 2026-10-07, `cargo xtask ios-build --simulator --release` and `cargo xtask ios-build --device --release` passed and produced unsigned bundles at `target/ios-minimal/simulator/ios-minimal.app` and `target/ios-minimal/device/ios-minimal.app`. Both executables imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`, with no Swift or Python runtime; `plutil -lint` passed for `target/ios-minimal/simulator/ios-minimal.app/Info.plist` and `target/ios-minimal/device/ios-minimal.app/Info.plist`. These are example builds under Xcode 26.6, not validation against the planned Xcode 27.x baseline. No simulator launch, signing, archive, or physical-device validation was performed. Use `cargo xtask toolchain-manifest` to record the active environment; see [Validation and Tooling](VALIDATION.md) for shared checks and scope.

## Goal

Keep application/framework source Rust-first while still using Apple's required build, signing, entitlement, asset, and App Store packaging toolchain.

## Expected toolchain split

Cargo/Rust is responsible for:

- framework/application Rust compilation;
- static libraries or other native artifacts;
- Rust dependencies;
- Rust tests and benchmarks.

Xcode/Apple tooling remains responsible where required for:

- application bundle construction;
- code signing;
- provisioning;
- entitlements;
- Info.plist integration;
- assets;
- device/simulator deployment;
- archive/export;
- App Store submission.

Using Xcode does not imply using Swift.

## No Swift build requirement

The framework must not require a Swift source compilation step.

CI should fail if repository or generated sources contain `.swift`.

## Initial vertical-slice requirement

Prove a Rust-owned application entry can reach:

```text
UIApplication
 -> Rust-defined app delegate
 -> UIWindow
 -> UIViewController
 -> UILabel + UIButton
 -> Rust callback
```

without Swift application logic.

If a tiny non-Swift linker or startup shim is required by Apple/Xcode mechanics, it must be documented and minimized.

## Architectures

At minimum support:

- arm64 iOS device;
- arm64 iOS simulator where supported by the development environment.

Additional simulator architectures should be added only if needed.

## Release validation

Keep exact commands in this document once the first Xcode project/workspace exists.

Required categories:

- Cargo workspace checks;
- Release device build;
- Release simulator build;
- Xcode bundle build;
- signing/archive smoke test when practical;
- zero-Swift-source check.
