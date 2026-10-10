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

## D46 execution evidence — 2026-10-10

The assigned baseline was `b5bdc50173ca86823c2cdfd5f210123b772f5314`. The status API, its guide,
and canonical row 088 already matched this bounded contract, so no implementation change was
needed. `framework_payments::ios::can_make_payments` is iOS-only, returns `false` below the iOS
10.0 floor, and makes only the `PKPaymentAuthorizationController::canMakePayments` query. Row 088
remains platform-exclusive and partial, with PassKit and Foundation listed and minimum iOS 10.0.

`sh crates/framework-payments/scripts/check.sh` passed on Rust 1.94.1, Xcode 26.6 (17F113), and
iOS SDK 26.5. Host/device/Simulator checks, strict Clippy, host and device rustdoc, source guards,
exact device/Simulator imports, docs-check, zero-Swift-source, and diff checks passed. The gate
built and inspected both arm64 link probes but did not execute either probe; no tests, live Apple
Pay query, or transaction behavior were exercised. Xcode 27.x qualification and physical-device
runtime evidence remain unverified. The pinned `ios-rust-validate` 0.1.0 profile list contains no
Apple Pay profile, so the package-local B51 gate remains the applicable validation.
