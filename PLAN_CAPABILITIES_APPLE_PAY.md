# D46: Apple Pay capability status

## Scope

Row 088 is platform-exclusive. D46 adds no portable payments contract. It owns one Boolean status
query at `framework_payments::ios::can_make_payments`, with the iOS implementation in B51

The query uses `PKPaymentAuthorizationController::canMakePayments` only. It reports general device
capability subject to hardware and parental controls. It does not check payment cards, merchant
networks, account state, app-specific access, payment authorization, transaction state, or payment
completion

## Acceptance

- `framework-payments` exists and exposes `ios::can_make_payments` only on iOS
- The status call returns `false` below the iOS 10.0 API floor
- No portable contract, payment UI, payment request, card query, merchant-network query, StoreKit,
  transaction callback, or payment processing is present
- Row 088 records the partial status-only scope and its exact framework and iOS floor
- The B51 device, Simulator, Clippy, rustdoc, source-guard, import, docs, zero-Swift, and diff gates
  pass without test or probe execution
