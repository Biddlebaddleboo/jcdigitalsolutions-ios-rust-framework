#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A legacy StoreKit purchase-ability status query for iOS"]

#[allow(deprecated)]
use objc2_store_kit::SKPaymentQueue;

/// Read Apple's legacy StoreKit purchase-ability status
///
/// This calls only `SKPaymentQueue::canMakePayments`. It does not create a payment queue, show
/// payment UI, request a purchase, read a product or account, or inspect a transaction or
/// entitlement
///
/// Apple deprecated this method at iOS 18.0 and lists StoreKit 2 `AppStore.canMakePayments` as an
/// alternative. The Rust API is deprecated too and exists only for legacy compatibility
///
/// This returns `false` below iOS 3.0. A `true` result is only the system purchase-ability bit; it
/// does not promise that a product exists or that a payment can complete
#[deprecated(
    note = "Apple deprecated SKPaymentQueue.canMakePayments in iOS 18; use StoreKit 2 AppStore.canMakePayments where available"
)]
#[allow(deprecated)]
pub fn legacy_can_make_payments() -> bool {
    if !objc2::available!(ios = 3.0, ..) {
        return false;
    }

    // SAFETY: the method takes no pointers or objects; this branch checks its iOS 3.0 API floor
    unsafe { SKPaymentQueue::canMakePayments() }
}
