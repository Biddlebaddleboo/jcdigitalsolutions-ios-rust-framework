# iOS StoreKit 2 purchase status

`ios-storekit2-status` exposes `ios_storekit2_status::can_make_payments() -> bool`

The function returns StoreKit's `AppStore.canMakePayments` value. `true` means StoreKit reports that the person can authorize purchases. A `false` result may reflect Screen Time or MDM purchase restrictions. It does not prove that a product is available or that a transaction will succeed

The StoreKit 2 property and symbol are available from iOS 15.0. The device and Simulator Release link probes use the Rust target defaults of minos 10.0 and 14.0, respectively. The StoreKit framework and property symbol are weak imports. The C thunk checks for a null weak symbol and returns `false` without a call when it is absent. Compiler IR and link inspection confirm this fallback path; no pre-iOS-15 runtime check was made. No product, transaction, account, entitlement, network, or UI API is part of this crate. Non-iOS targets return `false`

The implementation and IR proof target Xcode 26.6 / Swift 6.3.3 / SDK 26.5 only; that Xcode version is below the repository plan baseline. No live purchase or runtime query was made

See [D58](../../PLAN_CAPABILITIES_STOREKIT2_STATUS.md), [B63](../../PLAN_IOS_STOREKIT2_STATUS.md), and [B63 validation](../../PLAN_VALIDATION_IOS_STOREKIT2_STATUS.md)
