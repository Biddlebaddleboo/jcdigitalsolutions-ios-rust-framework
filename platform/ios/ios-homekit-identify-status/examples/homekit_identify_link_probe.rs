#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
fn main() {
    let probe: fn(
        &ios_homekit_identify_status::HMAccessory,
    ) -> Result<bool, ios_homekit_identify_status::HomeKitIdentifyStatusError> =
        ios_homekit_identify_status::supports_identify;
    core::hint::black_box(probe);
    let identify: fn(
        &ios_homekit_identify_status::HMAccessory,
    ) -> ios_homekit_identify_status::HomeKitIdentifyFuture = ios_homekit_identify_status::identify;
    core::hint::black_box(identify);
    core::hint::black_box(
        poll_identify as fn(&ios_homekit_identify_status::HMAccessory, &std::task::Waker),
    );
}

#[cfg(all(target_os = "ios", not(target_abi = "macabi")))]
fn poll_identify(accessory: &ios_homekit_identify_status::HMAccessory, waker: &std::task::Waker) {
    use core::future::Future;

    let mut future = std::pin::pin!(ios_homekit_identify_status::identify(accessory));
    let mut context = std::task::Context::from_waker(waker);
    let _ = future.as_mut().poll(&mut context);
}

#[cfg(not(all(target_os = "ios", not(target_abi = "macabi"))))]
fn main() {}
