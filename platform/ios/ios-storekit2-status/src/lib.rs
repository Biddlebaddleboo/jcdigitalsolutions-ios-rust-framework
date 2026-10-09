#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A small iOS StoreKit 2 purchase-ability status API"]

#[cfg(target_os = "ios")]
#[link(name = "StoreKit", kind = "framework")]
unsafe extern "C" {
    fn framework_storekit2_can_make_payments() -> bool;
}

/// Return StoreKit 2 purchase ability from `AppStore.canMakePayments`
///
/// `true` reports that the person can authorize purchases. `false` may reflect
/// purchase restrictions or an absent weak StoreKit symbol. This value does not
/// report product availability, account identity, entitlement state, or a successful transaction
///
/// On iOS this calls the public Swift property through one SDK-derived `swiftcall` C thunk
/// The API floor is iOS 15.0. No product, transaction, payment UI, account, or network API is used
///
/// On non-iOS targets this function returns `false`
pub fn can_make_payments() -> bool {
    #[cfg(target_os = "ios")]
    {
        // SAFETY: the C thunk uses the SDK-exported StoreKit symbol as a weak import and returns
        // false when that symbol has no definition
        unsafe { framework_storekit2_can_make_payments() }
    }

    #[cfg(not(target_os = "ios"))]
    {
        false
    }
}
