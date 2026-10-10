#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
fn main() {
    let probe: fn(
        &ios_homekit_identify_status::HMAccessory,
    ) -> Result<bool, ios_homekit_identify_status::HomeKitIdentifyStatusError> =
        ios_homekit_identify_status::supports_identify;
    core::hint::black_box(probe);
}

#[cfg(not(all(target_os = "ios", not(target_abi = "macabi"))))]
fn main() {}
