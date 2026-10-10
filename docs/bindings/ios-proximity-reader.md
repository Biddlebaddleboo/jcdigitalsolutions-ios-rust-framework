# iOS ProximityReader C ABI

F18 adds the opt-in `ios-proximity-reader` feature to `framework-c-api`. Its one synchronous query wraps B76's `ios_proximity_reader::tap_to_pay_device_model_supported()` result

`framework_ios_proximity_reader_tap_to_pay_device_model_supported` writes exactly 0 or 1 and returns `FRAMEWORK_STATUS_OK` on an iOS target whose deployment floor is 15.4. It reports only Apple's `PaymentCardReader.isSupported` device-model predicate: iPhone XS or newer. It does not check the operating-system version, entitlement, merchant approval, region, account, payment service provider, reader setup, NFC, or transaction readiness. It does not create a reader, start a session, show UI, or process a payment

The required `uint8_t` output must point to valid, properly aligned writable memory for the full synchronous call. The caller must prevent unsynchronized access to the output. The wrapper does not prove memory validity or retain the output address; it initializes the byte to zero before the query. A null output returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`; non-iOS returns `FRAMEWORK_STATUS_UNSUPPORTED`; an absent backend value maps to `FRAMEWORK_STATUS_UNAVAILABLE`; a caught Rust panic maps to `FRAMEWORK_STATUS_PANIC`. No callback, handle, native object, or owned buffer crosses C

The Swift-only getter is called by B76's compiler-checked C `swiftcall` thunk. F18 does not call or reproduce Swift ABI symbols. The focused C ABI gate links and inspects C11/C++17 device and Simulator probes at minos 15.4 and never executes them. See [D94](../../PLAN_CAPABILITIES_PROXIMITYREADER.md), [B76](../../PLAN_IOS_PROXIMITYREADER.md), [G92](../../PLAN_VALIDATION_IOS_PROXIMITYREADER.md), and [F18](../../PLAN_BINDINGS_COMPLETED_C_ABI.md)
