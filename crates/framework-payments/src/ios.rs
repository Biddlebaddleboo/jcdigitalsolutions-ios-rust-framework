use objc2_pass_kit::PKPaymentAuthorizationController;

/// Reads `PKPaymentAuthorizationController::canMakePayments`
///
/// The result reports general device capability, subject to hardware and parental controls. A
/// `true` result does not mean a payment card is set up or that a particular payment can complete
///
/// This returns `false` below iOS 10.0. It does not make a request, inspect cards or merchant
/// networks, create or present payment UI, authorize, or process a transaction
pub fn can_make_payments() -> bool {
    if !objc2::available!(ios = 10.0, ..) {
        return false;
    }

    // SAFETY: this branch enforces the iOS 10.0 class API floor; the query has no pointer inputs
    unsafe { PKPaymentAuthorizationController::canMakePayments() }
}
