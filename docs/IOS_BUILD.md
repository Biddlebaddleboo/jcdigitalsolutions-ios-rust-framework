# iOS Build and Packaging Direction

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
