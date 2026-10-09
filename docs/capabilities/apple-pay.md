# Apple Pay status

D46/B51 adds one iOS-only `can_make_payments` status query under
`framework_payments::ios`. It does not add a portable payments contract

The result reports general device capability, subject to hardware and parental controls. It does
not report card setup, merchant-network support, app-specific access, transaction approval, or
payment completion

See the [iOS Apple Pay status guide](../ios/apple-pay-availability.md) for the API floor, import
scope, and package checks
