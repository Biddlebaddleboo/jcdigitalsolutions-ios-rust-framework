# PLAN_IOS_PHOTOS.md — Workstream B27: PhotoKit Authorization Backend

## Status

B27 source, native-status mapping tests, guide, and local link/import script are implemented. In isolated temporary workspace `/tmp/photos-ios-check.zHPcdc`, host/device/Simulator checks, strict Clippy, rustdoc, and the exact link/import probe pass on Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5. No live app or authorization UI check is in scope

## Objective

Implement D22's read/write authorization query and explicit request with Apple's public PhotoKit access-level API

## Dependencies

- D22 `framework-photos`
- Exact `objc2-photos` 0.3.2 package pin with only `PHPhotoLibrary` and `block2` features; direct `block2` 0.6.2 with `alloc` supplies the callback block
- Xcode iOS SDK declares `PHAccessLevel`, `authorizationStatusForAccessLevel:`, `requestAuthorizationForAccessLevel:handler:`, and `PHAuthorizationStatusLimited` as iOS 14 APIs; backend deployment floor is iOS 14.0
- The dependency is package-local. Root owns `Cargo.lock` integration

## Write scope

- `PLAN_IOS_PHOTOS.md`
- `platform/ios/ios-photos/**`
- `docs/ios/photos.md`

Root owns workspace and lock integration, CI, canonical capability status, aggregate plans, shared docs indexes, and aggregate validation docs. Do not edit those shared paths or D21/B26/G20 files

## Required implementation

- Implement `PhotoLibraryAuthorizationBackend` with a stateless `IosPhotosBackend`
- Use `PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite)` for status queries
- Use `PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(PHAccessLevel::ReadWrite, ...)` for requests; do not use deprecated no-access-level APIs because they collapse Limited into Authorized
- Start the native request on first poll; use synchronized callback state safe for a callback queue not owned by Rust
- Accept one terminal callback, wake only after releasing locks, and suppress the Rust result after future drop; do not claim the native request or system UI is cancelled
- Map all public statuses and map unknown future values to `Unknown`; preserve Limited as distinct from Authorized
- Do not add asset fetch, image request, edits, PhotosUI, a picker, C ABI, global executor, Swift source, or permission/runtime claims
- Document the exact host `NSPhotoLibraryUsageDescription` Info.plist requirement for this read/write access scope; do not write or inspect host app settings

## Validation

- Run format, host tests, iOS device and Simulator target check, strict Clippy, rustdoc, and `sh -n`
- Run `platform/ios/ios-photos/check-link-imports.sh`; it builds but does not run device/Simulator probes and checks exact Photos/Foundation imports, symbols, and iOS 14.0 minos
- Run `git diff --check`
- Do not claim live UI, host plist configuration, authorization outcome, or asset access

## Apple API basis

- [PHPhotoLibrary](https://developer.apple.com/documentation/photos/phphotolibrary)
- [PHAuthorizationStatus](https://developer.apple.com/documentation/photos/phauthorizationstatus)
- [NSPhotoLibraryUsageDescription](https://developer.apple.com/documentation/bundleresources/information-property-list/nsphotolibraryusagedescription)

The active Xcode 26.6 / iOS SDK 26.5 `PHPhotoLibrary.h` marks `PHAccessLevel`, both access-level authorization methods, and `PHAuthorizationStatusLimited` as iOS 14 APIs. The Objective-C floor fixture compiles this header with an iOS 14.0 target and asserts the public ReadWrite value and all five public authorization-status values

## Validation record

In isolated workspace `/tmp/photos-ios-check.zHPcdc`, these checks pass on Rust 1.94.1:

- `cargo +1.94.1 fmt --manifest-path platform/ios/ios-photos/Cargo.toml -- --check`
- `cargo +1.94.1 test --locked --offline -p ios-photos` — four tests pass: two native-status mapping checks and two production completion-cell lifecycle checks
- `cargo +1.94.1 check --locked --offline -p ios-photos --target aarch64-apple-ios`
- `cargo +1.94.1 check --locked --offline -p ios-photos --target aarch64-apple-ios-sim`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-photos -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-photos --target aarch64-apple-ios -- -D warnings`
- `cargo +1.94.1 clippy --locked --offline --all-targets -p ios-photos --target aarch64-apple-ios-sim -- -D warnings`
- `cargo +1.94.1 doc --locked --offline -p ios-photos --no-deps --target aarch64-apple-ios`
- `cargo +1.94.1 tree --locked --offline --target aarch64-apple-ios -e features -p ios-photos`
- `sh platform/ios/ios-photos/check-link-imports.sh`
- Objective-C PhotoKit header floor fixture compile on device and Simulator iOS 14.0 targets

The dependency tree selects only `objc2-photos` 0.3.2 features `PHPhotoLibrary` and `block2`; direct `block2` 0.6.2 enables `alloc`. The link/import script reports exactly `Photos`, `Foundation`, `libSystem.B.dylib`, and `libobjc.A.dylib`; it verifies Objective-C class/message lookup symbols, rejects out-of-scope symbols and Swift runtime symbols, and confirms device and Simulator minos 14.0. Probe binaries are built but never run

The first shared-root offline resolver attempt returned `no matching package named objc2-photos found`. A standalone workspace resolved and downloaded the exact 0.3.2 package from crates.io and used its own Cargo.lock; root later integrated the shared lock entry. No workspace, lock, CI, index, or aggregate plan file was edited by B27
