# PLAN_IOS_PROXIMITYREADER.md — B76: ProximityReader device-model status

## Objective

Add one bounded iOS Rust query for `PaymentCardReader.isSupported`, the ProximityReader device-model predicate used by Tap to Pay on iPhone. This is not a Tap to Pay readiness or payment API

## Scope

- Package: `platform/ios/ios-proximity-reader`
- Public Rust API: `tap_to_pay_device_model_supported() -> Option<bool>`
- iOS floor: 15.4
- Framework: `ProximityReader`
- Supported deployment targets: arm64 iOS device and arm64 iOS Simulator
- Non-iOS targets return `None`
- No portable contract, permission prompt, reader creation, session, NFC access, payment processing, or Swift source

The Boolean reports only the device-model predicate. Apple documents true for iPhone XS or newer. It does not report OS-version support, entitlement, approved merchant/account, supported region, certified payment service provider, reader availability, or transaction readiness. Actual Tap to Pay use requires Apple's approved entitlement and organization account plus a participating certified payment service provider; those requirements do not make this predicate a readiness check

## ABI boundary

The installed public ProximityReader Swift interface declares `PaymentCardReader.isSupported` as a Swift-only static property and provides no Objective-C declaration or typed `objc2` binding. The implementation uses a C `swiftcall` thunk for the public Swift class metadata accessor and Boolean getter. Exact symbols:

- `_$s15ProximityReader011PaymentCardB0CMa`
- `_$s15ProximityReader011PaymentCardB0C11isSupportedSbvgZ`

The Swift metadata accessor returns the compiler-lowered `{ ptr, i64 }` response. The getter receives the metadata pointer as `swiftself` and returns `i1`; the thunk returns a fixed-width `uint8_t`. `check-swiftcall.sh` compares a temporary Swift oracle with Clang-lowered C calls for arm64 device and arm64 Simulator. The Swift oracle is created under the system temporary directory and is not shipped

## Validation

Run `sh platform/ios/ios-proximity-reader/check.sh` on macOS with Xcode and the iOS Rust targets installed. The focused gate passed on Xcode 26.6 build 17F113 / iOS SDK 26.5 and Rust 1.94.1:

- host check, strict Clippy, and rustdoc passed
- arm64 iOS device and Simulator checks and strict Clippy passed
- Swift/Clang ABI oracle matched on device and Simulator
- Release link probes imported ProximityReader and libSystem, retained the two public Swift symbols, and declared minos 15.4 on both targets
- link probes were inspected, not executed; no live device-model value was observed
- no tests ran

Xcode 26.6 / SDK 26.5 remains below the plan's Xcode 27.x baseline. This gate proves compiler-lowering, target compilation, and link shape for the local toolchain only; it does not prove runtime predicate results, payment readiness, entitlement, merchant setup, region, or PSP configuration

See [D94's API audit](PLAN_CAPABILITIES_PROXIMITYREADER.md), [G92](PLAN_VALIDATION_IOS_PROXIMITYREADER.md), and [the iOS guide](docs/ios/proximity-reader-status.md)
