# `ios-proximity-reader`

This crate exposes one non-prompting query: `tap_to_pay_device_model_supported()` calls Apple's
public `PaymentCardReader.isSupported` property and returns its Boolean value on iOS 15.4 or newer.
Non-iOS targets return `None`.

Apple documents the value as a device-model check (iPhone XS or newer), not an operating-system
version or Tap to Pay readiness result. It does not check the consuming app's entitlement, merchant
approval, region, payment service provider, or account, and it does not create a reader, start a
session, access NFC, or process a payment. Actual Tap to Pay use remains subject to Apple's
entitlement and participating payment-service-provider requirements.

The property is Swift-only in the installed SDK and has no Objective-C declaration or typed
`objc2` binding. The backend uses a minimal C `swiftcall` thunk for the public Swift metadata
accessor and getter. Its 64-bit metadata response and getter context are checked against temporary
Swift compiler-oracle output for device and Simulator targets. The oracle is created outside the
repository, no Swift source is shipped, and neither the gate nor link probe executes an artifact.

Run `sh platform/ios/ios-proximity-reader/check.sh` on macOS with Xcode and the Rust device/Simulator
targets installed. The gate checks the host fallback, target compilation, compiler-derived ABI
lowering, and Release link imports at the iOS 15.4 deployment floor.
