# iOS Apple Pay capability status

`framework_payments::ios::can_make_payments` calls the public
`PKPaymentAuthorizationController::canMakePayments` predicate

The class has an iOS 10.0 floor. The Rust API returns `false` below that floor. The predicate reports
general device capability, subject to hardware and parental controls. Apple notes that the result
can be `true` even when no payment card is set up

This package does not inspect cards or merchant networks, create a payment request, make or present
payment UI, authorize a payment, or process a transaction. It makes no claim about app-specific
payment access, card setup, merchant eligibility, transaction approval, or payment completion

The only PassKit API is `PKPaymentAuthorizationController::canMakePayments`. The package uses
`objc2-pass-kit` 0.3.2 with default features off and only
`PKPaymentAuthorizationController` enabled. Its expected direct imports are PassKit, Foundation,
`libobjc.A.dylib`, and `libSystem.B.dylib`; UIKit and StoreKit are outside scope

Apple API sources: [canMakePayments](https://developer.apple.com/documentation/passkit/pkpaymentauthorizationcontroller/canmakepayments%28%29),
[PKPaymentAuthorizationController](https://developer.apple.com/documentation/passkit/pkpaymentauthorizationcontroller)

## Package check gate

Run `sh crates/framework-payments/scripts/check.sh` from the repository root. The gate checks host,
device, and Simulator builds; strict Clippy; rustdoc; exact link imports; source scope; docs; zero
Swift source; and the diff. It does not run tests or invoke the link probe. Target checks need Xcode
and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` Rust targets
