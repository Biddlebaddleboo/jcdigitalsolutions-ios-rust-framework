# iOS Build and Packaging Direction

## Current build status

The architecture plan names Xcode 27.x as its execution-host baseline. The recorded host has Xcode 26.6 (build 17F113) and SDK 26.5, so it is below that baseline. The repository has a Cargo-based minimal iOS example and an Xcode archive project for that example, but no full framework Xcode project/workspace or iOS framework crate. On 2026-10-07, `cargo xtask ios-build --simulator --release`, `cargo xtask ios-build --device --release`, and `cargo xtask archive-smoke` passed. The archive smoke produced an unsigned `.xcarchive` at `target/ios-minimal/archive/ios-minimal.xcarchive`; `codesign -dv` reported `code object is not signed at all`. The archived executable imported UIKit, Foundation, CoreFoundation, `libobjc.A`, and `libSystem`, with no Swift or Python runtime, and `plutil -lint` passed for the archive and app `Info.plist` files. The iOS example builds/archive use Xcode 26.6 and do not meet the planned Xcode 27.x baseline. No simulator launch, signing/provisioning, archive export, installation, or physical-device validation was performed. Use `cargo xtask toolchain-manifest` to record the active environment; see [Validation and Tooling](VALIDATION.md) for shared checks and scope.

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

Required categories:

- Cargo workspace checks;
- Release device build;
- Release simulator build;
- Xcode bundle/archive smoke test;
- zero-Swift-source check.

## Unsigned Xcode archive smoke

Run on macOS with Xcode, the iPhoneOS SDK, Cargo, and the `aarch64-apple-ios` Rust target available:

```bash
cargo xtask archive-smoke
```

The command builds the Release device executable and app bundle with `examples/ios-minimal/build.sh device`, then runs the Xcode `ios-minimal` scheme's standard archive action:

```bash
xcodebuild \
  -project examples/ios-minimal/ios-minimal.xcodeproj \
  -scheme ios-minimal \
  -configuration Release \
  -destination 'generic/platform=iOS' \
  -derivedDataPath target/ios-minimal/archive-derived-data \
  -archivePath target/ios-minimal/archive/ios-minimal.xcarchive \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  CODE_SIGN_IDENTITY= \
  archive
```

The project stages the Cargo-built Rust executable and uses the existing `Info.plist`; the command verifies the archived executable/plists, checks Mach-O imports for Swift/Python runtimes, scans the archive for `.swift` files, and confirms that the app is unsigned. The archive is packaging/linkage evidence only. It does not provide signing, provisioning, archive export, App Store validation, simulator launch, installation, or physical-device validation. Signing/export remains a separate certificate-dependent gate.
