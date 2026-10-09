# CloudKit account-status contract

`framework-cloud` defines a portable, allocation-free `#![no_std]` contract for one account-status snapshot. It has no CloudKit dependency and does not expose Apple framework types. The separate [iOS adapter](../ios/cloudkit-account-status.md) calls the public CloudKit status API.

## Rust API

```rust
use framework_cloud::{AccountStatus, CloudAccount};
use ios_cloud::IosCloudAccountBackend;

async fn account_status() -> Result<AccountStatus, framework_core::Error> {
    let mut cloud = CloudAccount::new(IosCloudAccountBackend::new());
    let snapshot = cloud.account_status().await?;
    if let Some(error) = snapshot.error() {
        // The native callback may include both a status and an error.
        let _native_error = error;
    }
    Ok(snapshot.status())
}
```

`CloudAccount<B>` owns the caller-provided backend. `CloudAccountBackend` selects a concrete associated `core::future::Future` type statically; no boxed trait object, executor, `Send` requirement, global registry, or hidden initialization is imposed. The facade and snapshot are Rust-owned; the portable crate has no allocator or third-party dependency.

## Status and error semantics

`AccountStatus` distinguishes `CouldNotDetermine`, `Available`, `Restricted`, `NoAccount`, and `TemporarilyUnavailable`; `Unknown(i64)` preserves an unrecognized fixed-width native value rather than guessing. Apple introduced `CKAccountStatusTemporarilyUnavailable` in iOS 15.0; the status query API itself has an iOS 8.0 floor.

`AccountStatusSnapshot` is copyable and contains the status plus an optional `framework_core::Error`. The native API may report a status and an error in the same completion. The iOS adapter keeps a representable native error code and portable category only; it does not retain `NSError`, its domain, or description. An outer `Err` from the future is reserved for an operation-level backend failure.

`Availability` describes the backend/API context, not the user's account status or entitlement validity. An `Available` adapter result is not evidence that an iCloud account is signed in or that a container/database can be accessed.

## Operation and ownership

The operation starts when its future is first polled. A backend must deliver at most one terminal result while the future is live. Dropping before first poll starts no backend work. The facade does not require native cancellation: a backend must document whether drop cancels the operation or only abandons Rust interest. The iOS CloudKit query cannot be cancelled after it starts; its callback must remain safe and discard its result after future drop.

A snapshot reports status at query time only. It does not subscribe to account-change notifications, observe future sign-in/out changes, return account names or identifiers, or prove access to CloudKit records, databases, or any other data. Database, record, query, sync, subscription, and account-change APIs are out of scope.

## Support and evidence

D47 is a semantic contract, not a Rust replacement for CloudKit. The iOS path remains a partial Apple-backed implementation. It requires a configured CloudKit container and signed app entitlements; no live account probe, Apple parity test, or performance measurement is claimed. See [B52's iOS guide](../ios/cloudkit-account-status.md) for the native API, entitlements, callback queue, cancellation, and validation limits.
