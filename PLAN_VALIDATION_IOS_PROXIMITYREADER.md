# PLAN_VALIDATION_IOS_PROXIMITYREADER.md — G92: B76 ProximityReader

## Gate

`sh platform/ios/ios-proximity-reader/check.sh`

## Result

Passed on Xcode 26.6 build 17F113, iPhoneOS/iPhoneSimulator SDK 26.5, and Rust 1.94.1. Host check, strict Clippy, and rustdoc passed. Arm64 device and Simulator `cargo check` and strict Clippy passed. The temporary Swift compiler-oracle and Clang `swiftcall` thunk matched on both targets. Release link probes retained the public ProximityReader metadata accessor and `PaymentCardReader.isSupported` imports, loaded ProximityReader and libSystem, and reported minos 15.4 for device and Simulator

The linked probes were inspected but not executed. No tests, live device-model query, payment session, entitlement check, merchant/region/PSP check, or performance measurement ran. The result does not claim Tap to Pay readiness or full ProximityReader support. Xcode 26.6 / SDK 26.5 is below the Xcode 27.x plan baseline
