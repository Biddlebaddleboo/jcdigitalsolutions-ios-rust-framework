# iOS CloudKit account status

`ios-cloud` implements the portable [`framework-cloud` contract](../capabilities/cloudkit-account-status.md) with `CKContainer.accountStatus(completionHandler:)` through the generated `objc2-cloud-kit` 0.3.2 bindings. It queries only account status on the app's default container. It does not read or write CloudKit data.

## Use

```rust
use framework_cloud::{AccountStatus, CloudAccount};
use ios_cloud::IosCloudAccountBackend;

async fn read_cloud_account_status() -> Result<AccountStatus, framework_core::Error> {
    let mut cloud = CloudAccount::new(IosCloudAccountBackend::new());
    let snapshot = cloud.account_status().await?;
    if let Some(error) = snapshot.error() {
        // Inspect the framework-owned category/code if the native callback included an error.
        let _error = error;
    }
    Ok(snapshot.status())
}
```

The future starts the request on first poll. The adapter creates no global container or service registry. The completion arrives on a CloudKit-selected queue; the API does not promise the main queue. The callback copies the raw status and optional numeric error code into Rust-owned scalar values before it returns. Waker invocation occurs after the callback state lock is released. The callback is guarded against duplicate completion and Rust panic.

Dropping the future abandons Rust interest and clears its stored result and waker. CloudKit offers no cancellation operation for this query, so the native request may finish later; its retained callback then discards the result safely. A callback error is represented inside `AccountStatusSnapshot::error()` alongside the returned status. The adapter retains only the portable error category and representable numeric code, not the `NSError` object, domain, or description. No thread or executor is mandated for calling the Rust facade; CloudKit owns the completion queue.

## Status mapping and limits

The native enum values 0 through 4 map to `CouldNotDetermine`, `Available`, `Restricted`, `NoAccount`, and `TemporarilyUnavailable`; future/unrecognized raw values become `Unknown(i64)`. The `accountStatus` API is available from iOS 8.0. Apple marks `TemporarilyUnavailable` available from iOS 15.0. This is a declaration-derived API floor, not the installed SDK's deployment target.

An `Available` `CloudAccountBackend::availability()` result means this iOS backend's API surface is present. It does not inspect app signing configuration, prove that the default container is configured, report the current account status, or establish that database operations will succeed. A missing/mismatched container entitlement or CloudKit configuration can still cause a native query error.

This one-time snapshot does not listen for `CKAccountChangedNotification`, request sign-in, return account identity, or establish data access. No database, record, query, subscription, sync, or background behavior is implemented. `CKContainer`, `CKDatabase`, and native account/error objects are not exposed as escape handles.

## App configuration

The consuming app must enable iCloud with the CloudKit service and configure its container identifier. The signed app requires the iCloud container identifiers entitlement `com.apple.developer.icloud-container-identifiers` and `com.apple.developer.icloud-services` with the CloudKit service value. This account-status query requires no CloudKit-specific `Info.plist` usage-description key and does not display a privacy prompt. The query's status is not a permission grant for accessing a particular database.

## SDK, dependency, and validation evidence

The local Xcode 26.6 build 17F113 / iPhoneOS 26.5 SDK declares the account-status method at iOS 8.0 and the `TemporarilyUnavailable` status at iOS 15.0. This host is below the repository's Xcode 27.x baseline. The binding uses `objc2-cloud-kit` 0.3.2 with `default-features = false` and the `CKContainer` and `block2` features, plus `block2` 0.6.2 with `alloc`; `objc2-foundation` enables only `NSError`. Its generated method signature is `unsafe fn accountStatusWithCompletionHandler(&self, completion_handler: &block2::DynBlock<dyn Fn(CKAccountStatus, *mut NSError)>)`; the unsafe binding contract requires a sendable escaping block. `block2::RcBlock` owns the closure, which captures only an `Arc<Mutex<...>>`; the callback copies values immediately.

Device/simulator target checks and the package-local import gate compile/link a minimal consumer and inspect `otool -L`. Do not run the generated example: it starts an actual status query if executed. These gates do not exercise a live account, entitlement, CloudKit service response, simulator lifecycle, parity, or performance.

Apple references: [`CKContainer.accountStatus`](https://developer.apple.com/documentation/cloudkit/ckcontainer/accountstatus%28completionhandler%3A%29), [`CKAccountStatus`](https://developer.apple.com/documentation/cloudkit/ckaccountstatus), [`TemporarilyUnavailable`](https://developer.apple.com/documentation/cloudkit/ckaccountstatus/temporarilyunavailable), [Enabling CloudKit](https://developer.apple.com/documentation/cloudkit/enabling-cloudkit-in-your-app), [`com.apple.developer.icloud-container-identifiers`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.icloud-container-identifiers), and [`com.apple.developer.icloud-services`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.icloud-services).
