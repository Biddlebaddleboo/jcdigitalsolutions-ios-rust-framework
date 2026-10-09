# ProximityReader device-model status

`ios-proximity-reader::tap_to_pay_device_model_supported()` returns `Some(bool)` on iOS 15.4 or
newer and `None` on non-iOS targets. It calls only Apple's public `PaymentCardReader.isSupported`
property. Apple documents this as a device-model predicate: iPhone XS or newer

The result does not check the operating-system version, consuming-app entitlement, merchant
approval, region, account, payment service provider, reader, or transaction readiness. It does not
create a reader, start a session, access NFC, or process a payment. Actual Tap to Pay use is subject
to Apple's approved entitlement and organization account plus a participating certified payment
service provider

The Swift-only property is called through a compiler-derived C `swiftcall` thunk for the public
metadata accessor and getter. Device and arm64 Simulator compiler oracles and release link/import
checks are in `sh platform/ios/ios-proximity-reader/check.sh`; link probes are build-only and are not
executed
