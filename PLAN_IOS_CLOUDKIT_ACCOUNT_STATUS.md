# PLAN_IOS_CLOUDKIT_ACCOUNT_STATUS.md — Workstream B52: iOS CloudKit Account-Status Adapter

## Objective

Implement the D47 one-shot account-status contract with the public CloudKit `CKContainer.accountStatus` API. Do not access CloudKit data, prompt for account sign-in, or observe later account changes.

## API audit

The installed Xcode 26.6 / iPhoneOS 26.5 SDK declares `-[CKContainer accountStatusWithCompletionHandler:]` available from iOS 8.0. `CKAccountStatusTemporarilyUnavailable` is available from iOS 15.0. `objc2-cloud-kit` 0.3.2 exposes the generated `CKContainer` API behind the `CKContainer` and `block2` features. Its generated method takes `&block2::DynBlock<dyn Fn(CKAccountStatus, *mut NSError)>` and is `unsafe` because the escaping completion block must be sendable. The implementation uses `block2::RcBlock` with an owned `Arc<Mutex<...>>` capture and copies callback values before return. The required Xcode 27.x plan baseline is not met by this host.

Primary Apple references:

- [`CKContainer.accountStatus(completionHandler:)`](https://developer.apple.com/documentation/cloudkit/ckcontainer/accountstatus%28completionhandler%3A%29)
- [`CKAccountStatus`](https://developer.apple.com/documentation/cloudkit/ckaccountstatus)
- [`CKAccountStatus.temporarilyUnavailable`](https://developer.apple.com/documentation/cloudkit/ckaccountstatus/temporarilyunavailable)
- [Enabling CloudKit](https://developer.apple.com/documentation/cloudkit/enabling-cloudkit-in-your-app)

## Write scope

- `platform/ios/ios-cloud/**`
- `docs/ios/cloudkit-account-status.md`
- this plan
- the package-local `link_check` example and import-gate script

The isolated D47 workstream also updates the workspace dependency/lock and row 076; no other capability row is owned. Do not edit D43/D44 code in the shared checkout, add unrelated frameworks, or add Swift source.

## Required implementation

- Add an iOS-only `ios-cloud` crate implementing the D47 backend through generated public bindings.
- Use the app's default `CKContainer`; do not expose the container, database, account identity, native error object, or CloudKit dependency type through `framework-cloud`.
- Start the native request on first future poll. Copy status and optional error code into owned values inside the callback and never retain callback-borrowed `NSError` memory.
- Handle a callback from an arbitrary CloudKit queue with safe owned state, exactly-once terminal delivery, and reentrancy-safe waker behavior.
- Dropping the Rust future abandons interest and clears Rust result/waker state. It cannot cancel the already-started CloudKit account-status query; a late callback is discarded safely.
- Map raw status values 0 through 4 to the known semantic cases and preserve other values as `Unknown(i64)`. Preserve a representable native error code as a framework `PlatformErrorCode`; do not claim native domain/description preservation.
- Do not query a permission prompt or claim that the returned status proves database access. The API adapter's availability result does not check signing entitlements or the current iCloud account.
- Document CloudKit entitlement configuration: `com.apple.developer.icloud-container-identifiers` and `com.apple.developer.icloud-services` configured for CloudKit. No CloudKit-specific `Info.plist` usage-description key or permission prompt is required for this status query.

## Validation

- `cargo check --locked -p ios-cloud --target aarch64-apple-ios`
- `cargo check --locked -p ios-cloud --target aarch64-apple-ios-sim`
- strict all-target Clippy for both targets
- Build, but do not execute, the `link_check` example for each target; inspect imports with `otool -L`
- `cargo fmt --all -- --check`, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff --check`
- Do not add or run tests, launch a simulator, sign an app, or probe a live account in this slice.

## Acceptance boundary

The adapter is complete when device and simulator targets compile/link, expected CloudKit/Foundation imports are present, unrelated capability and Swift runtime imports are absent, and the portable status/error/cancellation contract is documented. These checks do not establish live account status, entitlement configuration, CloudKit data access, Apple parity, or performance.

## Integrated root validation (2026-10-08)

After merging into the existing iCloud identity `framework-cloud` and `ios-cloud` packages,
`sh platform/ios/ios-cloud/check-cloudkit.sh` passed on Rust 1.94.1 / Xcode 26.6 / SDK 26.5. This
includes locked portable and device/Simulator checks, strict Clippy, both non-executed link/import
probes, formatting, `cargo xtask docs-check`, `cargo xtask zero-swift-source`, and `git diff
--check`. No tests were added or run. No live CloudKit query or CI workflow run is claimed.
