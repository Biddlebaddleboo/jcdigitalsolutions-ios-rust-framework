#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Non-prompting iOS Tap to Pay device-model support query."]

/// Returns whether the current iPhone model supports Tap to Pay on iPhone.
///
/// On iOS, this calls only `PaymentCardReader.isSupported`, available from iOS 15.4. The value
/// identifies supported device models (iPhone XS or newer); it does not check the operating-system
/// version, app entitlement, merchant approval, region, payment service provider, or payment
/// readiness. It does not create a reader, start a session, access NFC, or process a payment.
/// Non-iOS targets return `None`.
pub fn tap_to_pay_device_model_supported() -> Option<bool> {
    #[cfg(target_os = "ios")]
    {
        // The C shim is built only for supported 64-bit Apple iOS targets and uses the exact
        // Swift ABI lowering verified by the package's compiler-oracle gate.
        let supported =
            unsafe { framework_proximity_reader_tap_to_pay_device_model_supported() != 0 };
        Some(supported)
    }

    #[cfg(not(target_os = "ios"))]
    {
        None
    }
}

#[cfg(target_os = "ios")]
unsafe extern "C" {
    fn framework_proximity_reader_tap_to_pay_device_model_supported() -> u8;
}
